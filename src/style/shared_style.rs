use crate::state::PseudoState;

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Margin, Rect, Shape, Stroke, Vec2, Visuals,
    pos2,
    style::WidgetVisuals,
};

/// How a `background_image` fills its container rect when the image's intrinsic
/// aspect ratio differs from the styled box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackgroundImageFit {
    /// Map the full texture over the rect (uv 0→1 on both axes). Distorts if
    /// aspect ratios differ — equivalent to CSS `background-size: 100% 100%`.
    #[default]
    Stretch,
    /// Scale to cover the box, cropping overflow. Preserves aspect ratio —
    /// equivalent to CSS `background-size: cover`. Requires the texture to be
    /// loaded; on the loading frame the image is not painted.
    Cover,
}

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

/// Compute the UV rect used to sample the texture for the given `fit` mode.
///
/// `intrinsic`: the loaded texture's original size in points (width, height).
/// `dest`: the target rect we are filling.
///
/// `Stretch` always returns `(0,0)→(1,1)`.
/// `Cover` adjusts the UV to crop the wider axis, preserving aspect ratio.
fn cover_uv(intrinsic: Vec2, dest: Rect) -> Rect {
    if intrinsic.x <= 0.0 || intrinsic.y <= 0.0 {
        return Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    }
    let dest_aspect = dest.width() / dest.height();
    let img_aspect = intrinsic.x / intrinsic.y;
    if img_aspect > dest_aspect {
        // image wider than dest — crop sides
        let scale = dest_aspect / img_aspect;
        let pad = (1.0 - scale) / 2.0;
        Rect::from_min_max(pos2(pad, 0.0), pos2(1.0 - pad, 1.0))
    } else {
        // image taller than dest — crop top/bottom
        let scale = img_aspect / dest_aspect;
        let pad = (1.0 - scale) / 2.0;
        Rect::from_min_max(pos2(0.0, pad), pos2(1.0, 1.0 - pad))
    }
}

/// Build the `Shape` that paints a background image clipped to a rounded rect.
///
/// Returns `None` when the texture is not yet loaded (async loading path — the
/// caller leaves its `Shape::Noop` placeholder in place and the image paints on
/// the next frame).
///
/// Layers (front-to-back inside the returned `Shape::Vec`):
/// 1. `bg` fill (if any) — solid colour behind the texture
/// 2. Texture rectangle (rounded, tinted)
/// 3. Border stroke on top of the texture
pub fn background_image_shape(
    ui: &egui::Ui,
    rect: Rect,
    corner_radius: CornerRadius,
    image: &egui::Image<'static>,
    fit: BackgroundImageFit,
    tint: Color32,
    bg: Option<Color32>,
    border: Option<Stroke>,
    fade: Option<(egui::Id, f32)>,
) -> Option<Shape> {
    use egui::load::TexturePoll;
    use egui::epaint::RectShape;

    let poll = image
        .load_for_size(ui.ctx(), rect.size())
        .ok()?;

    let texture = match poll {
        TexturePoll::Ready { texture } => texture,
        TexturePoll::Pending { .. } => return None,
    };

    let tint = match fade {
        Some((id, duration)) if duration > 0.0 => {
            let now = ui.ctx().input(|i| i.time);
            let start = ui.ctx().memory_mut(|m| {
                *m.data.get_temp_mut_or_insert_with(id, || now)
            });
            let alpha = (((now - start) / duration as f64).clamp(0.0, 1.0)) as f32;
            if alpha < 1.0 {
                ui.ctx().request_repaint();
            }
            tint.gamma_multiply(alpha)
        }
        _ => tint,
    };

    let uv = match fit {
        BackgroundImageFit::Stretch => Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        BackgroundImageFit::Cover => cover_uv(texture.size, rect),
    };

    let mut parts: Vec<Shape> = Vec::with_capacity(3);

    if let Some(fill) = bg {
        parts.push(Shape::rect_filled(rect, corner_radius, fill));
    }

    let tex_shape = RectShape::filled(rect, corner_radius, tint)
        .with_texture(texture.id, uv);
    parts.push(Shape::Rect(tex_shape));

    if let Some(stroke) = border {
        parts.push(Shape::rect_stroke(
            rect,
            corner_radius,
            stroke,
            egui::StrokeKind::Outside,
        ));
    }

    Some(Shape::Vec(parts))
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

    // Background image — drawn on top of `bg` fill, clipped to the same rounded rect.
    // Texture loading is the consuming app's responsibility (egui loader registry or
    // `ctx.load_texture`). egui_styled only paints a texture it has already received.
    pub background_image: Option<egui::Image<'static>>,
    pub background_image_fit: BackgroundImageFit,
    pub background_image_tint: Option<Color32>,
    /// Opt-in fade-in duration (seconds). When `Some`, the image fades up from
    /// the bg backdrop over the given duration once its texture is Ready.
    /// `None` = no fade (snap in, default behavior).
    pub background_image_fade_in: Option<f32>,
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
            || self.background_image.is_some()
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

    #[test]
    fn background_image_fit_default_is_stretch() {
        assert_eq!(BackgroundImageFit::default(), BackgroundImageFit::Stretch);
    }

    #[test]
    fn has_frame_styles_triggered_by_background_image() {
        let style = SharedStyle {
            background_image: Some(egui::Image::from_bytes("bytes://test", vec![])),
            ..Default::default()
        };
        assert!(style.has_frame_styles());
    }

    #[test]
    fn cover_uv_stretch_returns_full() {
        let dest = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(100.0, 100.0));
        let uv = cover_uv(egui::Vec2::new(200.0, 100.0), dest);
        // Stretch isn't tested via cover_uv; Cover is. Wider image into square dest:
        // img_aspect=2, dest_aspect=1 → scale=0.5, pad=0.25 on x axis
        let uv_cover = cover_uv(egui::Vec2::new(200.0, 100.0), dest);
        assert!((uv_cover.min.x - 0.25).abs() < 1e-5);
        assert!((uv_cover.max.x - 0.75).abs() < 1e-5);
        assert!((uv_cover.min.y - 0.0).abs() < 1e-5);
        assert!((uv_cover.max.y - 1.0).abs() < 1e-5);
        let _ = uv;
    }

    #[test]
    fn cover_uv_tall_image_crops_vertically() {
        // 100×200 image into 100×100 dest → taller image, crop top/bottom
        let dest = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(100.0, 100.0));
        let uv = cover_uv(egui::Vec2::new(100.0, 200.0), dest);
        assert!((uv.min.x - 0.0).abs() < 1e-5);
        assert!((uv.max.x - 1.0).abs() < 1e-5);
        // img_aspect=0.5, dest_aspect=1 → scale=0.5, pad=0.25 on y axis
        assert!((uv.min.y - 0.25).abs() < 1e-5);
        assert!((uv.max.y - 0.75).abs() < 1e-5);
    }

    #[test]
    fn cover_uv_square_image_returns_full() {
        let dest = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(100.0, 100.0));
        let uv = cover_uv(egui::Vec2::new(100.0, 100.0), dest);
        assert!((uv.min.x - 0.0).abs() < 1e-5);
        assert!((uv.max.x - 1.0).abs() < 1e-5);
        assert!((uv.min.y - 0.0).abs() < 1e-5);
        assert!((uv.max.y - 1.0).abs() < 1e-5);
    }
}
