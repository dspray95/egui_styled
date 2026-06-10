use crate::state::PseudoState;

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Margin, Rect, Shape, Stroke, Vec2, Visuals, pos2,
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

/// Per-side border overrides. Each side is `None` until the user sets it via a
/// `border_top` / `border_left` / … builder; an unset side falls back to the
/// uniform `border` (and then egui's default) at resolve time.
#[derive(Clone, Copy, Default, Debug)]
pub struct SideStrokes {
    pub top: Option<Stroke>,
    pub right: Option<Stroke>,
    pub bottom: Option<Stroke>,
    pub left: Option<Stroke>,
}

impl SideStrokes {
    /// True if any side has an explicit override.
    pub fn any(&self) -> bool {
        self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some()
    }
}

/// A border resolved to a concrete `Stroke` per side. Produced by
/// [`SharedStyle::resolve`]; consumed by [`paint_side_borders`] when the user
/// set per-side overrides.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedBorder {
    pub top: Stroke,
    pub right: Stroke,
    pub bottom: Stroke,
    pub left: Stroke,
}

/// Paint each border side as a straight line segment along the matching edge of
/// `rect`. Only sides with a positive stroke width are drawn.
///
/// Unlike egui's uniform `bg_stroke`, partial borders are **not** rounded around
/// the corner radius — each side is a straight edge. This is fine for the common
/// case (left/right or top/bottom borders) and keeps the painting trivial.
pub fn paint_side_borders(painter: &egui::Painter, rect: Rect, border: ResolvedBorder) {
    let line = |a, b, stroke: Stroke| {
        if stroke.width > 0.0 {
            painter.add(Shape::line_segment([a, b], stroke));
        }
    };
    line(rect.left_top(), rect.right_top(), border.top);
    line(rect.right_top(), rect.right_bottom(), border.right);
    line(rect.left_bottom(), rect.right_bottom(), border.bottom);
    line(rect.left_top(), rect.left_bottom(), border.left);
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

/// Fade alpha for a background-image reveal. Stamps the first-`ready` time in
/// `ctx` memory under `id`, so the image tint and (optionally) the body content
/// fade in lockstep off the same clock. Returns 1.0 when `duration <= 0`, and
/// 0.0 while `ready` is false — without stamping, so the clock only starts once
/// the texture has actually landed.
/// Vertically distribute body content within a container that has a known height.
///
/// Adds a top spacer of `(fill_height - cached_content_h) * justify_factor`,
/// renders the body (invisibly on the first frame when no height is cached yet),
/// then writes the measured content height back to memory and requests a repaint
/// on that measurement frame so the content appears correctly positioned
/// immediately on the next frame without a visible pop.
///
/// - `fill_height`: the container's total available height (e.g. `screen_size.y`
///   or `ui.available_height()` before sizing calls).
/// - `justify_factor`: 0.0 = top, 0.5 = center, 1.0 = bottom (from
///   [`egui::Align::to_factor`]).
/// - `content_h_id`: a stable [`egui::Id`] used to persist the measured height
///   between frames. Derive with `ui.make_persistent_id(...).with("some_tag")`.
pub fn justify_body_vertically<R>(
    ui: &mut egui::Ui,
    fill_height: f32,
    justify_factor: f32,
    content_h_id: egui::Id,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let cached_h = ui.ctx().memory(|m| m.data.get_temp::<f32>(content_h_id));
    let top_pad = ((fill_height - cached_h.unwrap_or(0.0)) * justify_factor).max(0.0);
    ui.add_space(top_pad);
    let scope = ui.scope(|ui| {
        if cached_h.is_none() {
            ui.set_invisible();
        }
        body(ui)
    });
    let measured_h = scope.response.rect.height();
    ui.ctx()
        .memory_mut(|m| m.data.insert_temp(content_h_id, measured_h));
    if cached_h.is_none() {
        ui.ctx().request_repaint();
    }
    scope.inner
}

/// How to distribute children horizontally across available width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distribution {
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Compute leading space `L` and inter-item gap `G` for a distribution mode.
///
/// Returns `(L, G)`. When `slack <= 0` or `n == 0`, falls back to `(0, min_gap)`.
pub fn distribution_spacing(mode: Distribution, slack: f32, n: usize, min_gap: f32) -> (f32, f32) {
    if n == 0 {
        return (0.0, min_gap);
    }
    if slack <= 0.0 {
        return (0.0, min_gap);
    }
    let nf = n as f32;
    let (l, g) = match mode {
        Distribution::SpaceBetween => {
            if n <= 1 {
                (0.0, 0.0)
            } else {
                (0.0, slack / (nf - 1.0))
            }
        }
        Distribution::SpaceAround => (slack / (2.0 * nf), slack / nf),
        Distribution::SpaceEvenly => (slack / (nf + 1.0), slack / (nf + 1.0)),
    };
    (l, g.max(min_gap))
}

/// Lay out a row of items with CSS-style distribution using a cross-frame W cache.
///
/// On the first frame (no cached W), renders items invisibly with zero spacing to
/// measure content width, caches it, and requests a repaint. On subsequent frames,
/// uses the cached W to compute leading space and inter-item gap per `mode`.
#[allow(clippy::too_many_arguments)]
pub fn distribute_row_horizontally(
    ui: &mut egui::Ui,
    avail: f32,
    mode: Distribution,
    min_gap: f32,
    n: usize,
    cross_align: egui::Align,
    content_w_id: egui::Id,
    render_items: impl FnOnce(&mut egui::Ui),
) {
    let cached_w = ui.ctx().memory(|m| m.data.get_temp::<f32>(content_w_id));
    let layout = egui::Layout::left_to_right(cross_align);
    // Seed with a one-row height so a `left_to_right(Center)` layout does not
    // balloon to fill the parent's full height (see the note in `StyledRow`).
    let initial_size = egui::vec2(avail, ui.spacing().interact_size.y);

    if let Some(w) = cached_w {
        let slack = (avail - w).max(0.0);
        let (leading, gap) = distribution_spacing(mode, slack, n, min_gap);

        ui.allocate_ui_with_layout(initial_size, layout, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if leading > 0.0 {
                ui.add_space(leading);
            }
            ui.spacing_mut().item_spacing.x = gap;
            render_items(ui);
        });
    } else {
        // Measure frame: invisible, zero spacing, record content width.
        let scope = ui.scope(|ui| {
            ui.set_invisible();
            ui.allocate_ui_with_layout(initial_size, layout, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                render_items(ui);
                ui.min_rect().width()
            })
            .inner
        });
        let measured_w = scope.inner;
        ui.ctx()
            .memory_mut(|m| m.data.insert_temp(content_w_id, measured_w));
        ui.ctx().request_repaint();
    }
}

/// Fade alpha for a background-image reveal. Stamps the first-`ready` time in
pub fn bgimg_fade_alpha(ctx: &egui::Context, id: egui::Id, duration: f32, ready: bool) -> f32 {
    if duration <= 0.0 {
        return 1.0;
    }
    if !ready {
        return 0.0;
    }
    let now = ctx.input(|i| i.time);
    let start = ctx.memory_mut(|m| *m.data.get_temp_mut_or_insert_with(id, || now));
    let alpha = (((now - start) / duration as f64).clamp(0.0, 1.0)) as f32;
    if alpha < 1.0 {
        ctx.request_repaint();
    }
    alpha
}

/// Build the `Shape` that paints a background image clipped to a rounded rect.
///
/// Returns `None` only when there is nothing to draw (no `bg`, no `border`, and
/// texture not yet ready). The `bg` fill and `border` paint every frame; the
/// textured layer is added on top once the texture is `Ready`.
///
/// Layers (front-to-back inside the returned `Shape::Vec`):
/// 1. `bg` fill (if any) — solid colour behind the texture
/// 2. Texture rectangle (rounded, tinted) — omitted while still loading
/// 3. Border stroke on top
#[allow(clippy::too_many_arguments)]
pub fn background_image_shape(
    ui: &egui::Ui,
    rect: Rect,
    corner_radius: CornerRadius,
    image: &egui::Image<'static>,
    fit: BackgroundImageFit,
    tint: Color32,
    bg: Option<Color32>,
    border: Option<Stroke>,
    border_sides: Option<ResolvedBorder>,
    fade: Option<(egui::Id, f32)>,
) -> Option<Shape> {
    use egui::epaint::RectShape;
    use egui::load::TexturePoll;

    let mut parts: Vec<Shape> = Vec::with_capacity(3);

    // Bg fill paints every frame from the first — does not wait for the texture.
    if let Some(fill) = bg {
        parts.push(Shape::rect_filled(rect, corner_radius, fill));
    }

    // Probe the texture; only add the textured layer when it is Ready.
    let poll = image.load_for_size(ui.ctx(), rect.size()).ok();
    if let Some(TexturePoll::Ready { texture }) = poll {
        let tint = match fade {
            // We only reach here once the texture is Ready.
            Some((id, duration)) if duration > 0.0 => {
                tint.gamma_multiply(bgimg_fade_alpha(ui.ctx(), id, duration, true))
            }
            _ => tint,
        };

        let uv = match fit {
            BackgroundImageFit::Stretch => Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            BackgroundImageFit::Cover => cover_uv(texture.size, rect),
        };

        let tex_shape = RectShape::filled(rect, corner_radius, tint).with_texture(texture.id, uv);
        parts.push(Shape::Rect(tex_shape));
    }

    if let Some(sides) = border_sides {
        // Per-side overrides: paint each edge as a straight line segment, same
        // as `paint_side_borders`. Takes precedence over the uniform `border`.
        let mut push = |a, b, stroke: Stroke| {
            if stroke.width > 0.0 {
                parts.push(Shape::line_segment([a, b], stroke));
            }
        };
        push(rect.left_top(), rect.right_top(), sides.top);
        push(rect.right_top(), rect.right_bottom(), sides.right);
        push(rect.left_bottom(), rect.right_bottom(), sides.bottom);
        push(rect.left_top(), rect.left_bottom(), sides.left);
    } else if let Some(stroke) = border {
        parts.push(Shape::rect_stroke(
            rect,
            corner_radius,
            stroke,
            egui::StrokeKind::Outside,
        ));
    }

    if parts.is_empty() {
        None
    } else {
        Some(Shape::Vec(parts))
    }
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

    // Per-side border overrides. Each side falls back to the matching uniform
    // border above when unset. `border_sides` is the base state; the hover/focus
    // variants override it under those interaction states.
    pub border_sides: SideStrokes,
    pub hover_border_sides: SideStrokes,
    pub focus_border_sides: SideStrokes,

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
    pub full_height: bool,
    /// Width as a percentage (0–100) of the parent's available width.
    /// When set, resolves to a definite size and supersedes `full_width`.
    /// Composed with `min_width`/`max_width` as clamps after resolution.
    pub width_pct: Option<f32>,
    /// Height as a percentage (0–100) of the parent's available height.
    /// When set, resolves to a definite size and supersedes `full_height`.
    /// Composed with `min_height`/`max_height` as clamps after resolution.
    pub height_pct: Option<f32>,

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
    /// When `true`, the body content fades in together with the background image
    /// (same timing) instead of rendering immediately over the backdrop. Set via
    /// `reveal_with_background_image`. The `bg` fill stays opaque.
    pub background_image_fade_content: bool,
}

/// Concrete style values for one interaction state after resolving pseudo-state
/// and falling back to egui defaults.
#[derive(Clone)]
pub struct ResolvedStyle {
    pub bg: Color32,
    pub text_color: Color32,
    /// Uniform border representative — preserves existing behavior (egui
    /// `bg_stroke`, focus ring). Equal to the resolved uniform base stroke.
    pub border: Stroke,
    /// Full per-side border. Each side = override.or(uniform base).or(default).
    /// Only painted (via [`paint_side_borders`]) when `has_border_overrides`.
    pub border_sides: ResolvedBorder,
    /// True when this state has any per-side override set. Drives the switch
    /// between egui's uniform painting and manual per-side painting.
    pub has_border_overrides: bool,
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
    /// Resolve `width_pct` to a definite width in points, clamped by
    /// `min_width`/`max_width`. `avail` should be `ui.available_width()`
    /// captured before any sizing mutations. Returns `None` when unset.
    pub fn resolved_width_pct(&self, avail: f32) -> Option<f32> {
        self.width_pct.map(|p| {
            let mut w = avail * p / 100.0;
            if let Some(m) = self.max_width {
                w = w.min(m);
            }
            if let Some(m) = self.min_width {
                w = w.max(m);
            }
            w
        })
    }

    /// Resolve `height_pct` to a definite height in points, clamped by
    /// `min_height`/`max_height`. `avail` should be `ui.available_height()`
    /// captured before any sizing mutations. Returns `None` when unset.
    pub fn resolved_height_pct(&self, avail: f32) -> Option<f32> {
        self.height_pct.map(|p| {
            let mut h = avail * p / 100.0;
            if let Some(m) = self.max_height {
                h = h.min(m);
            }
            if let Some(m) = self.min_height {
                h = h.max(m);
            }
            h
        })
    }

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

        // Per-side overrides follow the same focus > hover > base precedence as
        // the uniform border. Each side falls back to `border` (the resolved
        // uniform stroke) when not explicitly overridden for this state.
        let sides = match state {
            _ if state.focused && self.focus_border_sides.any() => self.focus_border_sides,
            _ if state.hovered && self.hover_border_sides.any() => self.hover_border_sides,
            _ => self.border_sides,
        };
        let has_border_overrides = sides.any();
        let border_sides = ResolvedBorder {
            top: sides.top.unwrap_or(border),
            right: sides.right.unwrap_or(border),
            bottom: sides.bottom.unwrap_or(border),
            left: sides.left.unwrap_or(border),
        };

        let text_color = match state {
            _ if state.hovered && self.hover_text_color.is_some() => self.hover_text_color.unwrap(),
            _ => self.text_color.unwrap_or(default.text_color()),
        };

        ResolvedStyle {
            bg,
            text_color,
            border,
            border_sides,
            has_border_overrides,
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
            // When per-side overrides are present we paint the border ourselves
            // (egui's bg_stroke is uniform-only), so suppress egui's stroke.
            wv.bg_stroke = if resolved.has_border_overrides {
                Stroke::NONE
            } else {
                resolved.border
            };
            wv.corner_radius = per.corner_radius;
            wv.expansion = 0.0;
            wv.fg_stroke = Stroke::new(wv.fg_stroke.width, resolved.text_color);
        }
        // Also update the open state for combo-box menus.
        vis.widgets.open.bg_fill = per.inactive.bg;
        vis.widgets.open.weak_bg_fill = per.inactive.bg;
        vis.widgets.open.bg_stroke = if per.inactive.has_border_overrides {
            Stroke::NONE
        } else {
            per.inactive.border
        };
        vis.widgets.open.corner_radius = per.corner_radius;
        vis.widgets.open.expansion = 0.0;

        vis.extreme_bg_color = per.inactive.bg;
        vis.selection.bg_fill = per.accent;
        vis.selection.stroke = per.focused.border;
    }

    /// Pick the `ResolvedStyle` matching a widget response's interaction state,
    /// mirroring which `WidgetVisuals` slot egui itself would paint with.
    pub fn for_response<'a>(
        per: &'a PerStateStyle,
        response: &egui::Response,
    ) -> &'a ResolvedStyle {
        if response.has_focus() {
            &per.focused
        } else if response.is_pointer_button_down_on() {
            &per.active
        } else if response.hovered() {
            &per.hovered
        } else {
            &per.inactive
        }
    }

    /// Paint per-side borders for a widget when the response's interaction state
    /// has overrides set. No-op otherwise (egui already painted the uniform
    /// `bg_stroke`). Call after the widget renders, using its response rect.
    pub fn paint_widget_side_borders(
        ui: &egui::Ui,
        response: &egui::Response,
        per: &PerStateStyle,
    ) {
        let state = Self::for_response(per, response);
        if state.has_border_overrides {
            paint_side_borders(ui.painter(), response.rect, state.border_sides);
        }
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
            || self.border_sides.any()
            || self.hover_border_sides.any()
            || self.focus_border_sides.any()
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
    fn side_override_sets_only_that_side() {
        let left = Stroke::new(3.0, Color32::RED);
        let style = SharedStyle {
            border_sides: SideStrokes {
                left: Some(left),
                ..Default::default()
            },
            ..Default::default()
        };
        let visuals = default_visuals();
        let resolved = style.resolve(PseudoState::default(), &visuals);
        assert!(resolved.has_border_overrides);
        assert_eq!(resolved.border_sides.left, left);
        // Unset sides fall back to the uniform base (here egui's default stroke).
        assert_eq!(resolved.border_sides.top, visuals.bg_stroke);
        assert_eq!(resolved.border_sides.right, visuals.bg_stroke);
        assert_eq!(resolved.border_sides.bottom, visuals.bg_stroke);
    }

    #[test]
    fn side_override_falls_back_to_uniform_border() {
        let uniform = Stroke::new(1.0, Color32::GRAY);
        let left = Stroke::new(3.0, Color32::RED);
        let style = SharedStyle {
            border: Some(uniform),
            border_sides: SideStrokes {
                left: Some(left),
                ..Default::default()
            },
            ..Default::default()
        };
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert_eq!(resolved.border_sides.left, left);
        assert_eq!(resolved.border_sides.top, uniform);
        assert_eq!(resolved.border_sides.right, uniform);
        assert_eq!(resolved.border_sides.bottom, uniform);
        // The uniform representative is unchanged.
        assert_eq!(resolved.border, uniform);
    }

    #[test]
    fn no_side_override_means_no_overrides_flag() {
        let uniform = Stroke::new(1.0, Color32::GRAY);
        let style = SharedStyle {
            border: Some(uniform),
            ..Default::default()
        };
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert!(!resolved.has_border_overrides);
        assert_eq!(resolved.border, uniform);
    }

    #[test]
    fn focus_side_override_beats_base() {
        let base_left = Stroke::new(1.0, Color32::GRAY);
        let focus_left = Stroke::new(2.0, Color32::YELLOW);
        let style = SharedStyle {
            border_sides: SideStrokes {
                left: Some(base_left),
                ..Default::default()
            },
            focus_border_sides: SideStrokes {
                left: Some(focus_left),
                ..Default::default()
            },
            ..Default::default()
        };
        let focused = PseudoState {
            hovered: false,
            active: false,
            focused: true,
        };
        assert_eq!(
            style.resolve(focused, &default_visuals()).border_sides.left,
            focus_left
        );
        // Base state still uses the base side.
        assert_eq!(
            style
                .resolve(PseudoState::default(), &default_visuals())
                .border_sides
                .left,
            base_left
        );
    }

    #[test]
    fn paint_side_borders_only_emits_positive_width_sides() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let rect = Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(50.0, 30.0));
            // Only left + right have width; top + bottom are zero-width.
            let border = ResolvedBorder {
                top: Stroke::NONE,
                right: Stroke::new(2.0, Color32::RED),
                bottom: Stroke::new(0.0, Color32::RED),
                left: Stroke::new(2.0, Color32::RED),
            };
            paint_side_borders(ui.painter(), rect, border);
        });
        let segments = output
            .shapes
            .iter()
            .filter(|cs| matches!(cs.shape, Shape::LineSegment { .. }))
            .count();
        assert_eq!(segments, 2, "expected one segment per positive-width side");
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
            (
                "border_sides",
                SharedStyle {
                    border_sides: SideStrokes {
                        left: Some(Stroke::new(1.0, Color32::RED)),
                        ..Default::default()
                    },
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

    #[test]
    fn resolved_width_pct_none_when_unset() {
        let style = SharedStyle::default();
        assert!(style.resolved_width_pct(400.0).is_none());
    }

    #[test]
    fn resolved_width_pct_half_of_avail() {
        let style = SharedStyle { width_pct: Some(50.0), ..Default::default() };
        let w = style.resolved_width_pct(400.0).unwrap();
        assert!((w - 200.0).abs() < 1e-4, "50% of 400 = 200, got {w}");
    }

    #[test]
    fn resolved_width_pct_clamped_by_max_width() {
        let style = SharedStyle {
            width_pct: Some(50.0),
            max_width: Some(120.0),
            ..Default::default()
        };
        let w = style.resolved_width_pct(400.0).unwrap();
        assert!((w - 120.0).abs() < 1e-4, "50% of 400 clamped to max_width 120, got {w}");
    }

    #[test]
    fn resolved_width_pct_floored_by_min_width() {
        let style = SharedStyle {
            width_pct: Some(10.0),
            min_width: Some(80.0),
            ..Default::default()
        };
        let w = style.resolved_width_pct(400.0).unwrap();
        assert!((w - 80.0).abs() < 1e-4, "10% of 400=40 raised to min_width 80, got {w}");
    }

    #[test]
    fn resolved_height_pct_half_of_avail() {
        let style = SharedStyle { height_pct: Some(50.0), ..Default::default() };
        let h = style.resolved_height_pct(300.0).unwrap();
        assert!((h - 150.0).abs() < 1e-4, "50% of 300 = 150, got {h}");
    }

    // ── distribution_spacing pure-math tests ────────────────────────────────

    #[test]
    fn dist_between_equal_gaps() {
        // avail=400, W=100, n=3 → slack=300, L=0, G=150
        let (l, g) = distribution_spacing(Distribution::SpaceBetween, 300.0, 3, 0.0);
        assert!((l - 0.0).abs() < 1e-4, "between: L should be 0, got {l}");
        assert!((g - 150.0).abs() < 1e-4, "between: G should be 150, got {g}");
    }

    #[test]
    fn dist_around_equal_margin() {
        // slack=300, n=3 → L=50, G=100
        let (l, g) = distribution_spacing(Distribution::SpaceAround, 300.0, 3, 0.0);
        assert!((l - 50.0).abs() < 1e-4, "around: L should be 50, got {l}");
        assert!((g - 100.0).abs() < 1e-4, "around: G should be 100, got {g}");
    }

    #[test]
    fn dist_evenly_equal_everywhere() {
        // slack=400, n=3 → L=G=100
        let (l, g) = distribution_spacing(Distribution::SpaceEvenly, 400.0, 3, 0.0);
        assert!((l - 100.0).abs() < 1e-4, "evenly: L should be 100, got {l}");
        assert!((g - 100.0).abs() < 1e-4, "evenly: G should be 100, got {g}");
    }

    #[test]
    fn dist_single_item_centers_for_around_evenly() {
        // n=1, around/evenly: L = slack/2
        let (l_around, _) = distribution_spacing(Distribution::SpaceAround, 200.0, 1, 0.0);
        let (l_evenly, _) = distribution_spacing(Distribution::SpaceEvenly, 200.0, 1, 0.0);
        assert!((l_around - 100.0).abs() < 1e-4, "around n=1: L=100, got {l_around}");
        assert!((l_evenly - 100.0).abs() < 1e-4, "evenly n=1: L=100, got {l_evenly}");
    }

    #[test]
    fn dist_zero_slack_uses_min_gap() {
        let (l, g) = distribution_spacing(Distribution::SpaceBetween, 0.0, 3, 8.0);
        assert!((l - 0.0).abs() < 1e-4, "zero slack: L=0, got {l}");
        assert!((g - 8.0).abs() < 1e-4, "zero slack: G=min_gap=8, got {g}");
    }
}
