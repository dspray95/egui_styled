use crate::state::PseudoState;

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Margin, Shape, Stroke, Vec2, Visuals,
    style::WidgetVisuals,
};

/// A single paint decoration rendered behind the widget rect.
///
/// Use `.shadow()` (stroke-only) or `.shadow_filled()` (solid fill) on any
/// styled widget to paint offset copies of the widget's bounding rect on the
/// same layer, underneath the widget itself.
///
/// Multiple shadows are supported — append with repeated `.shadow(...)` calls.
/// Each shadow inherits the widget's `corner_radius` at paint time.
#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    pub offset: Vec2,
    pub stroke: Stroke,
    pub fill: Option<Color32>,
}

/// Paint all `shadows` behind position `reserve_idx` (a `Shape::Noop`
/// placeholder inserted before the widget was rendered). Replaces the
/// placeholder with a `Shape::Vec` containing all shadow shapes.
///
/// Call pattern inside a widget's `show`:
/// ```text
/// let shadow_idx = ui.painter().add(Shape::Noop);
/// // ... add widget, get response ...
/// paint_shadows(ui, shadow_idx, response.rect, corner_radius, &style.shadows);
/// ```
pub fn paint_shadows(
    ui: &egui::Ui,
    reserve_idx: egui::layers::ShapeIdx,
    rect: egui::Rect,
    corner_radius: CornerRadius,
    shadows: &[Shadow],
) {
    if shadows.is_empty() {
        return;
    }
    let shapes: Vec<Shape> = shadows
        .iter()
        .flat_map(|s| {
            let r = rect.translate(s.offset);
            let mut parts: Vec<Shape> = Vec::with_capacity(2);
            if let Some(fill) = s.fill {
                parts.push(Shape::rect_filled(r, corner_radius, fill));
            }
            if s.stroke != Stroke::NONE {
                parts.push(Shape::rect_stroke(
                    r,
                    corner_radius,
                    s.stroke,
                    egui::StrokeKind::Outside,
                ));
            }
            parts
        })
        .collect();
    ui.painter().set(reserve_idx, Shape::Vec(shapes));
}

/// Render `f` inside a child scope so a `visible == false` widget's
/// invisibility is contained to itself.
///
/// `Ui::set_invisible()` mutates the painter/enabled state of the `Ui` it's
/// called on and affects *all* subsequent widgets. Calling it on a cloned
/// child `Ui` (what `ui.scope` provides) keeps the effect local: the widget
/// hides but still allocates layout space, and following siblings are
/// unaffected. The scope must enclose the widget's entire visual output —
/// frame, shadows, and inner content — so call shadow allocation/painting
/// inside `f`.
pub fn render_scoped<R>(ui: &mut egui::Ui, visible: bool, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.scope(|ui| {
        if !visible {
            ui.set_invisible();
        }
        f(ui)
    })
    .inner
}

/// The style bag carried by every styled widget.
///
/// Every field is `Option<T>` so `None` falls through to egui's active
/// `Visuals` at resolve time - we never overwrite a default the user didn't
/// explicitly set. Widget-specific properties (button image, slider step,
/// etc.) live on the widget struct itself, not here.
///
/// You rarely construct this directly; the `impl_style_builders!` macro
/// generates `.bg()`, `.hover_bg()`, etc. on each styled type.
#[derive(Clone, Default, Debug)]
pub struct SharedStyle {
    // Background
    pub bg: Option<Color32>,
    pub hover_bg: Option<Color32>,
    pub active_bg: Option<Color32>,
    pub focus_bg: Option<Color32>,

    // Accent — maps to egui's `selection.bg_fill` / `selection.stroke`
    // (slider trailing fill, text-edit focus ring, text selection highlight).
    pub accent: Option<Color32>,
    pub hover_accent: Option<Color32>,

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

    // Visibility — `Some(false)` calls `Ui::set_invisible()`: reserves layout
    // space, skips painting, implies disabled (no hover/click interaction).
    // `None` and `Some(true)` are both fully visible.
    pub visible: Option<bool>,

    // Decorations
    pub shadows: Vec<Shadow>,
}

/// Concrete style values for one interaction state after resolving pseudo-state
/// and falling back to egui defaults.
#[derive(Clone)]
pub struct ResolvedStyle {
    pub bg: Color32,
    pub text_color: Color32,
    pub border: Stroke,
    pub corner_radius: CornerRadius,
    pub padding: Margin,
    pub margin: Margin,
    pub cursor_icon: Option<CursorIcon>,
}

/// Resolved style for all interaction states simultaneously. Allows writing
/// each per-state colour into the matching `WidgetVisuals` slot so egui's own
/// hover/active response picks the right variant without clobbering it.
#[derive(Clone)]
pub struct PerStateStyle {
    pub inactive: ResolvedStyle,
    pub hovered: ResolvedStyle,
    pub active: ResolvedStyle,
    /// Focused mirrors `focus_*` fields; falls back to `inactive` if unset.
    pub focused: ResolvedStyle,
    /// Maps to `selection.bg_fill` (slider trail, text selection).
    pub accent: Color32,
    /// Maps to `selection.bg_fill` on hover.
    pub hover_accent: Color32,
    /// Shared across states.
    pub corner_radius: CornerRadius,
    pub padding: Margin,
    pub margin: Margin,
    pub cursor_icon: Option<CursorIcon>,
}

impl SharedStyle {
    /// Resolve against current pseudo-state and egui's active visuals.
    /// Kept for back-compat; prefer `resolve_per_state` for new widget code.
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

    /// Resolve a `ResolvedStyle` for each interaction state independently.
    /// Use this in widget `show()` implementations so each state's colour is
    /// written into the matching `WidgetVisuals` slot.
    pub fn resolve_per_state(&self, visuals: &Visuals) -> PerStateStyle {
        let resolve_one = |state: PseudoState, wv: &WidgetVisuals| self.resolve(state, wv);

        let inactive = resolve_one(
            PseudoState {
                hovered: false,
                active: false,
                focused: false,
            },
            &visuals.widgets.inactive,
        );
        let hovered = resolve_one(
            PseudoState {
                hovered: true,
                active: false,
                focused: false,
            },
            &visuals.widgets.hovered,
        );
        let active = resolve_one(
            PseudoState {
                hovered: true,
                active: true,
                focused: false,
            },
            &visuals.widgets.active,
        );
        let focused = resolve_one(
            PseudoState {
                hovered: false,
                active: false,
                focused: true,
            },
            &visuals.widgets.inactive,
        );

        let corner_radius = self
            .corner_radius
            .unwrap_or(visuals.widgets.inactive.corner_radius);
        let accent = self.accent.unwrap_or(visuals.selection.bg_fill);
        let hover_accent = self.hover_accent.unwrap_or(accent);

        PerStateStyle {
            inactive,
            hovered,
            active,
            focused,
            accent,
            hover_accent,
            corner_radius,
            padding: self.padding.unwrap_or_default(),
            margin: self.margin.unwrap_or_default(),
            cursor_icon: self.cursor_icon,
        }
    }

    /// Write `per_state` into `vis` so every egui widget inside the current
    /// `ui.scope` inherits correct per-interaction-state colours.
    ///
    /// Handles all known egui visuals quirks:
    /// - `weak_bg_fill` (used by ComboBox button) is kept in sync with `bg_fill`
    /// - `expansion` is zeroed so border rects don't drift
    /// - `selection.bg_fill` receives `accent` (slider trail, text selection)
    /// - `selection.stroke` receives `focused.border` (text-edit focus ring)
    /// - `extreme_bg_color` receives `inactive.bg` (TextEdit background)
    pub fn apply_to_visuals(per: &PerStateStyle, vis: &mut Visuals) {
        let states: [(&ResolvedStyle, &mut WidgetVisuals); 3] = [
            (&per.inactive, &mut vis.widgets.inactive),
            (&per.hovered, &mut vis.widgets.hovered),
            (&per.active, &mut vis.widgets.active),
        ];
        for (resolved, wv) in states {
            wv.bg_fill = resolved.bg;
            wv.weak_bg_fill = resolved.bg;
            wv.bg_stroke = resolved.border;
            wv.corner_radius = per.corner_radius;
            wv.expansion = 0.0;
            wv.fg_stroke = Stroke::new(wv.fg_stroke.width, resolved.text_color);
        }
        // Also update the open state for combo-box menus.
        vis.widgets.open.bg_fill = per.inactive.bg;
        vis.widgets.open.weak_bg_fill = per.inactive.bg;
        vis.widgets.open.bg_stroke = per.inactive.border;
        vis.widgets.open.corner_radius = per.corner_radius;
        vis.widgets.open.expansion = 0.0;

        vis.extreme_bg_color = per.inactive.bg;
        vis.selection.bg_fill = per.accent;
        vis.selection.stroke = per.focused.border;
    }

    /// True if any field that an [`egui::Frame`] could render is set.
    ///
    /// Containers (`StyledRow`, `StyledColumn`, `StyledStack`) use this to
    /// decide whether to wrap themselves in a `StyledFrame` or render directly.
    /// `margin` counts because it is applied via the wrapper frame's
    /// `outer_margin` - a margin-only container must still route through the
    /// frame or the spacing is silently dropped. Text color and sizing are
    /// intentionally excluded - they are not "frame" concerns.
    pub fn has_frame_styles(&self) -> bool {
        self.bg.is_some()
            || self.hover_bg.is_some()
            || self.active_bg.is_some()
            || self.focus_bg.is_some()
            || self.border.is_some()
            || self.hover_border.is_some()
            || self.focus_border.is_some()
            || self.padding.is_some()
            || self.corner_radius.is_some()
            || self.margin.is_some()
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

    #[test]
    fn has_frame_styles_empty() {
        assert!(!SharedStyle::default().has_frame_styles());
    }

    #[test]
    fn has_frame_styles_each_trigger() {
        let triggers: Vec<(&str, SharedStyle)> = vec![
            (
                "bg",
                SharedStyle {
                    bg: Some(Color32::RED),
                    ..Default::default()
                },
            ),
            (
                "hover_bg",
                SharedStyle {
                    hover_bg: Some(Color32::RED),
                    ..Default::default()
                },
            ),
            (
                "active_bg",
                SharedStyle {
                    active_bg: Some(Color32::RED),
                    ..Default::default()
                },
            ),
            (
                "focus_bg",
                SharedStyle {
                    focus_bg: Some(Color32::RED),
                    ..Default::default()
                },
            ),
            (
                "border",
                SharedStyle {
                    border: Some(Stroke::new(1.0, Color32::RED)),
                    ..Default::default()
                },
            ),
            (
                "hover_border",
                SharedStyle {
                    hover_border: Some(Stroke::new(1.0, Color32::RED)),
                    ..Default::default()
                },
            ),
            (
                "focus_border",
                SharedStyle {
                    focus_border: Some(Stroke::new(1.0, Color32::RED)),
                    ..Default::default()
                },
            ),
            (
                "padding",
                SharedStyle {
                    padding: Some(egui::Margin::same(4)),
                    ..Default::default()
                },
            ),
            (
                "corner_radius",
                SharedStyle {
                    corner_radius: Some(egui::CornerRadius::same(4)),
                    ..Default::default()
                },
            ),
            (
                "margin",
                SharedStyle {
                    margin: Some(egui::Margin::same(4)),
                    ..Default::default()
                },
            ),
        ];
        for (name, style) in triggers {
            assert!(
                style.has_frame_styles(),
                "{name} did not trigger has_frame_styles"
            );
        }
    }

    #[test]
    fn has_frame_styles_ignores_non_frame_props() {
        // text_color and full_width are not frame concerns and must not flip
        // has_frame_styles. (margin does count - see has_frame_styles_each_trigger.)
        let style = SharedStyle {
            text_color: Some(Color32::RED),
            full_width: true,
            ..Default::default()
        };
        assert!(!style.has_frame_styles());
    }

    #[test]
    fn resolve_per_state_each_variant_independent() {
        let style = SharedStyle {
            bg: Some(Color32::RED),
            hover_bg: Some(Color32::GREEN),
            active_bg: Some(Color32::BLUE),
            focus_bg: Some(Color32::YELLOW),
            accent: Some(Color32::WHITE),
            ..Default::default()
        };
        let vis = Visuals::default();
        let per = style.resolve_per_state(&vis);
        assert_eq!(per.inactive.bg, Color32::RED);
        assert_eq!(per.hovered.bg, Color32::GREEN);
        assert_eq!(per.active.bg, Color32::BLUE);
        assert_eq!(per.focused.bg, Color32::YELLOW);
        assert_eq!(per.accent, Color32::WHITE);
    }

    #[test]
    fn resolve_per_state_accent_falls_back_to_selection_bg_fill() {
        let style = SharedStyle::default();
        let vis = Visuals::default();
        let per = style.resolve_per_state(&vis);
        assert_eq!(per.accent, vis.selection.bg_fill);
    }
}
