use crate::state::PseudoState;

use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Margin, Mesh, Rect, Shape, Stroke, Vec2, Visuals,
    pos2, style::WidgetVisuals,
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

/// A 4-corner gradient fill painted behind a widget.
///
/// Colors are bilinearly interpolated across the rect using a 2×2 GPU texture,
/// so `corner_radius` is respected (the tessellator remaps UVs per vertex along
/// rounded paths). Use [`BgGradient::vertical`] / [`BgGradient::horizontal`] for
/// the common 2-stop cases.
///
/// **Cache note:** each unique set of four colors allocates one 2×2 GPU texture
/// per egui `Context` for its lifetime. Keep gradient colors from a fixed palette
/// to avoid unbounded growth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BgGradient {
    pub top_left: Color32,
    pub top_right: Color32,
    pub bottom_left: Color32,
    pub bottom_right: Color32,
}

impl BgGradient {
    pub fn corners(tl: Color32, tr: Color32, bl: Color32, br: Color32) -> Self {
        Self {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br,
        }
    }
    /// Vertical two-stop gradient (top color → bottom color).
    pub fn vertical(top: Color32, bottom: Color32) -> Self {
        Self {
            top_left: top,
            top_right: top,
            bottom_left: bottom,
            bottom_right: bottom,
        }
    }
    /// Horizontal two-stop gradient (left color → right color).
    pub fn horizontal(left: Color32, right: Color32) -> Self {
        Self {
            top_left: left,
            top_right: right,
            bottom_left: left,
            bottom_right: right,
        }
    }
}

/// The axis a [`LinearGradient`] runs along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GradientAxis {
    /// Top → bottom.
    Vertical,
    /// Left → right.
    Horizontal,
}

/// A multi-stop linear gradient (e.g. a rainbow) along one axis.
///
/// Baked into a 256×1 (or 1×256) GPU texture and sampled with bilinear
/// filtering, so `corner_radius` is respected just like [`BgGradient`]. Stops are
/// `(position, color)` pairs with `position` in `0.0..=1.0`; they are sorted on
/// construction and clamped at the ends.
///
/// **Cache note:** each unique stop set allocates one ramp texture per egui
/// `Context` for its lifetime — keep stops from a fixed palette.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// `(position, color)` stops, sorted by position.
    pub stops: Vec<(f32, Color32)>,
    pub axis: GradientAxis,
}

impl LinearGradient {
    /// Build from arbitrary `(position, color)` stops along `axis`. Stops are
    /// sorted by position; positions outside `0..=1` are clamped at sample time.
    pub fn new(stops: impl IntoIterator<Item = (f32, Color32)>, axis: GradientAxis) -> Self {
        let mut stops: Vec<(f32, Color32)> = stops.into_iter().collect();
        stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Self { stops, axis }
    }

    /// Evenly-spaced stops along `axis` (first at 0.0, last at 1.0).
    pub fn evenly_spaced(colors: impl IntoIterator<Item = Color32>, axis: GradientAxis) -> Self {
        let colors: Vec<Color32> = colors.into_iter().collect();
        let n = colors.len();
        let stops = colors.into_iter().enumerate().map(|(i, c)| {
            let pos = if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            (pos, c)
        });
        Self::new(stops, axis)
    }

    /// Sample the gradient at `t` (0.0..=1.0), interpolating between stops in
    /// gamma space. Clamps to the first/last stop outside the stop range.
    pub fn sample(&self, t: f32) -> Color32 {
        match self.stops.as_slice() {
            [] => Color32::TRANSPARENT,
            [(_, c)] => *c,
            stops => {
                let t = t.clamp(0.0, 1.0);
                if t <= stops[0].0 {
                    return stops[0].1;
                }
                if t >= stops[stops.len() - 1].0 {
                    return stops[stops.len() - 1].1;
                }
                let hi = stops.iter().position(|s| s.0 >= t).unwrap();
                let (p0, c0) = stops[hi - 1];
                let (p1, c1) = stops[hi];
                let span = (p1 - p0).max(f32::EPSILON);
                c0.lerp_to_gamma(c1, (t - p0) / span)
            }
        }
    }
}

impl std::hash::Hash for LinearGradient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.axis.hash(state);
        self.stops.len().hash(state);
        for (pos, color) in &self.stops {
            pos.to_bits().hash(state);
            color.hash(state);
        }
    }
}

/// A background gradient — either a 4-corner bilinear blend ([`BgGradient`]) or a
/// multi-stop linear ramp ([`LinearGradient`]). Paints over the solid `bg` fill.
#[derive(Clone, Debug, PartialEq)]
pub enum Gradient {
    Corners(BgGradient),
    Linear(LinearGradient),
}

impl std::hash::Hash for Gradient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Gradient::Corners(g) => {
                0u8.hash(state);
                g.hash(state);
            }
            Gradient::Linear(g) => {
                1u8.hash(state);
                g.hash(state);
            }
        }
    }
}

/// An inward glow: bright at the rect edge, fading toward the center.
///
/// Rendered as concentric stroke rings using `StrokeKind::Inside`. The
/// `corner_radius` of the widget is inherited so the glow conforms to rounded
/// corners.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InnerGlow {
    /// Total glow depth in logical pixels (clamped to half the rect's min dimension).
    pub width: f32,
    /// Glow color; alpha fades to transparent toward the center.
    pub color: Color32,
    /// Which edges the glow is drawn from. [`Sides::ALL`] = a full ring that
    /// follows the corner radius; any partial selection draws straight bands
    /// from the chosen edges (corners are not rounded).
    pub sides: Sides,
}

impl InnerGlow {
    /// A full-ring glow (all four sides).
    pub fn new(width: f32, color: Color32) -> Self {
        Self {
            width,
            color,
            sides: Sides::ALL,
        }
    }

    /// A glow drawn only from the given `sides`.
    pub fn with_sides(width: f32, color: Color32, sides: Sides) -> Self {
        Self {
            width,
            color,
            sides,
        }
    }
}

/// A selection of the four rectangle edges, used by per-side [`InnerGlow`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Sides {
    /// All four edges.
    pub const ALL: Sides = Sides {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };
    /// Top and bottom edges only.
    pub const Y: Sides = Sides {
        top: true,
        right: false,
        bottom: true,
        left: false,
    };
    /// Left and right edges only.
    pub const X: Sides = Sides {
        top: false,
        right: true,
        bottom: false,
        left: true,
    };
    pub const TOP: Sides = Sides {
        top: true,
        right: false,
        bottom: false,
        left: false,
    };
    pub const RIGHT: Sides = Sides {
        top: false,
        right: true,
        bottom: false,
        left: false,
    };
    pub const BOTTOM: Sides = Sides {
        top: false,
        right: false,
        bottom: true,
        left: false,
    };
    pub const LEFT: Sides = Sides {
        top: false,
        right: false,
        bottom: false,
        left: true,
    };

    /// True when every edge is selected.
    pub fn is_all(&self) -> bool {
        self.top && self.right && self.bottom && self.left
    }
    /// True when at least one edge is selected.
    pub fn any(&self) -> bool {
        self.top || self.right || self.bottom || self.left
    }
}

/// A border whose color interpolates vertically between `top` and `bottom`.
///
/// Rendered as 4 mitered trapezoid meshes (no corner-radius support — same
/// accepted limitation as per-side borders). `border_gradient` takes precedence
/// over both uniform `border` and per-side border overrides for the same state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderGradient {
    /// Border thickness in logical pixels.
    pub width: f32,
    /// Color at the top edge of the rect.
    pub top: Color32,
    /// Color at the bottom edge of the rect.
    pub bottom: Color32,
}

impl BorderGradient {
    pub fn new(width: f32, top: Color32, bottom: Color32) -> Self {
        Self { width, top, bottom }
    }
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

/// Retrieve (or create and cache) the 2×2 gradient texture for `g`.
///
/// The texture is stored as a temp value in the egui `Context`'s `IdTypeMap`,
/// keyed by the four corner colors. Each unique color quad allocates one
/// 2×2 GPU texture per context lifetime; there is no eviction in v1.
///
/// **Deadlock note:** `ctx.load_texture` and `ctx.data_mut` both lock the same
/// `ContextImpl` RwLock (egui 0.34 — read lock vs write lock). Never call
/// `load_texture` from inside `data_mut`.
pub fn gradient_texture(ctx: &egui::Context, g: BgGradient) -> egui::TextureHandle {
    let key = egui::Id::new(("egui_styled::bg_gradient", g));
    if let Some(handle) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return handle;
    }
    // Load OUTSIDE data_mut to avoid deadlock.
    let img = egui::ColorImage::new(
        [2, 2],
        vec![g.top_left, g.top_right, g.bottom_left, g.bottom_right],
    );
    let handle = ctx.load_texture("egui_styled_bg_gradient", img, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, handle.clone()));
    handle
}

/// Build a `Shape` that fills `rect` (respecting `corner_radius`) with the
/// bilinear gradient described by `g`.
///
/// Uses a 2×2 texture with UVs at texel centers `(0.25,0.25)–(0.75,0.75)` so
/// the visible rect spans the full interpolation zone without clamped-edge bands.
pub fn bg_gradient_shape(
    ctx: &egui::Context,
    rect: Rect,
    cr: CornerRadius,
    g: BgGradient,
) -> Shape {
    use egui::epaint::RectShape;
    let tex = gradient_texture(ctx, g);
    let uv = Rect::from_min_max(pos2(0.25, 0.25), pos2(0.75, 0.75));
    Shape::Rect(RectShape::filled(rect, cr, Color32::WHITE).with_texture(tex.id(), uv))
}

/// Number of texels used to bake a [`LinearGradient`] ramp. High enough that
/// half-texel clamp at the ends is sub-pixel.
const LINEAR_RAMP_TEXELS: usize = 256;

/// Retrieve (or bake and cache) the ramp texture for a [`LinearGradient`].
///
/// Vertical gradients bake to a `1×N` column, horizontal to an `N×1` row. Same
/// deadlock-safe check-then-load pattern as [`gradient_texture`].
pub fn linear_gradient_texture(ctx: &egui::Context, g: &LinearGradient) -> egui::TextureHandle {
    let key = egui::Id::new(("egui_styled::linear_gradient", g));
    if let Some(handle) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return handle;
    }
    let n = LINEAR_RAMP_TEXELS;
    let pixels: Vec<Color32> = (0..n)
        .map(|i| g.sample((i as f32 + 0.5) / n as f32))
        .collect();
    let size = match g.axis {
        GradientAxis::Vertical => [1, n],
        GradientAxis::Horizontal => [n, 1],
    };
    let img = egui::ColorImage::new(size, pixels);
    let handle = ctx.load_texture(
        "egui_styled_linear_gradient",
        img,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|d| d.insert_temp(key, handle.clone()));
    handle
}

/// Build a `Shape` that fills `rect` (respecting `corner_radius`) with a
/// multi-stop linear gradient. UVs are inset by half a texel along the gradient
/// axis so the ramp spans the full interpolation zone without clamped-edge bands.
pub fn linear_gradient_shape(
    ctx: &egui::Context,
    rect: Rect,
    cr: CornerRadius,
    g: &LinearGradient,
) -> Shape {
    use egui::epaint::RectShape;
    let tex = linear_gradient_texture(ctx, g);
    let inset = 0.5 / LINEAR_RAMP_TEXELS as f32;
    let uv = match g.axis {
        GradientAxis::Vertical => Rect::from_min_max(pos2(0.0, inset), pos2(1.0, 1.0 - inset)),
        GradientAxis::Horizontal => Rect::from_min_max(pos2(inset, 0.0), pos2(1.0 - inset, 1.0)),
    };
    Shape::Rect(RectShape::filled(rect, cr, Color32::WHITE).with_texture(tex.id(), uv))
}

/// Build the fill `Shape` for any [`Gradient`], respecting `corner_radius`.
pub fn gradient_shape(ctx: &egui::Context, rect: Rect, cr: CornerRadius, g: &Gradient) -> Shape {
    match g {
        Gradient::Corners(c) => bg_gradient_shape(ctx, rect, cr, *c),
        Gradient::Linear(l) => linear_gradient_shape(ctx, rect, cr, l),
    }
}

/// Build a mitered "picture-frame" ring mesh between `outer` and `inner` rects.
///
/// Eight outer corner vertices (indices 0–7) and eight matching inner corner
/// vertices (indices 8–15) form four trapezoid quads (top / right / bottom /
/// left), mitered at 45° so adjacent quads share corner vertices without overlap.
/// Each vertex's color is chosen by `outer_color` / `inner_color`, and the GPU
/// interpolates linearly across every triangle — giving a smooth (band-free)
/// gradient across the ring band.
///
/// Corner radius is not respected (straight mitered edges).
fn ring_mesh(
    outer: Rect,
    inner: Rect,
    outer_color: impl Fn(egui::Pos2) -> Color32,
    inner_color: impl Fn(egui::Pos2) -> Color32,
) -> Mesh {
    let mut mesh = Mesh::default();

    // Outer corners (duplicated per shared edge so each quad owns its pair).
    //
    //  0---1
    //  |8-9|
    // 7|   |2
    // 6|   |3
    //  |15 10
    //  5---4
    let o = [
        outer.left_top(),
        outer.right_top(),
        outer.right_top(),
        outer.right_bottom(),
        outer.right_bottom(),
        outer.left_bottom(),
        outer.left_bottom(),
        outer.left_top(),
    ];
    let i = [
        inner.left_top(),
        inner.right_top(),
        inner.right_top(),
        inner.right_bottom(),
        inner.right_bottom(),
        inner.left_bottom(),
        inner.left_bottom(),
        inner.left_top(),
    ];

    for p in &o {
        mesh.colored_vertex(*p, outer_color(*p));
    }
    for p in &i {
        mesh.colored_vertex(*p, inner_color(*p));
    }

    // 4 trapezoid quads: top / right / bottom / left.
    // Each quad: outer[a], outer[b], inner[b], inner[a].
    let quads: [(u32, u32); 4] = [(0, 1), (2, 3), (4, 5), (6, 7)];
    for (a, b) in quads {
        let ia = a + 8;
        let ib = b + 8;
        mesh.add_triangle(a, b, ib);
        mesh.add_triangle(a, ib, ia);
    }

    mesh
}

/// Build a smooth ring mesh between a rounded outer outline (`rect` with corner
/// radius `cr`) and a concentric inner outline inset by `width`.
///
/// Both outlines are sampled at the **same** set of arc angles per corner, so
/// every outer vertex has a matching inner vertex and the band can be stripped
/// into quads. Each outer vertex is colored by `outer_color`, each inner vertex
/// by `inner_color`, and the GPU interpolates linearly across the band — no
/// banding, and the corners follow `cr`.
///
/// When `width >= corner_radius` the inner corner radius collapses to 0, so the
/// rounded outer corner fans smoothly into the sharp inset inner corner.
fn rounded_ring_mesh(
    rect: Rect,
    cr: CornerRadius,
    width: f32,
    outer_color: impl Fn(egui::Pos2) -> Color32,
    inner_color: impl Fn(egui::Pos2) -> Color32,
) -> Mesh {
    use std::f32::consts::{FRAC_PI_2, PI};

    let inner = rect.shrink(width);
    let max_r = (rect.width().min(rect.height()) * 0.5).max(0.0);

    // Per corner: the radius field, the outer/inner rect corners, and the arc's
    // start angle. Angles sweep clockwise in screen space (y down): a point on
    // the arc is `center + r * (cos a, sin a)`, and the arc center is the rect
    // corner pulled inward by the radius.
    let corners = [
        (
            cr.nw,
            rect.min.x,
            rect.min.y,
            inner.min.x,
            inner.min.y,
            1.0,
            1.0,
            PI,
        ),
        (
            cr.ne,
            rect.max.x,
            rect.min.y,
            inner.max.x,
            inner.min.y,
            -1.0,
            1.0,
            PI + FRAC_PI_2,
        ),
        (
            cr.se,
            rect.max.x,
            rect.max.y,
            inner.max.x,
            inner.max.y,
            -1.0,
            -1.0,
            PI + 2.0 * FRAC_PI_2,
        ),
        (
            cr.sw,
            rect.min.x,
            rect.max.y,
            inner.min.x,
            inner.max.y,
            1.0,
            -1.0,
            PI + 3.0 * FRAC_PI_2,
        ),
    ];

    let mut outer_pts: Vec<egui::Pos2> = Vec::new();
    let mut inner_pts: Vec<egui::Pos2> = Vec::new();
    for (r_field, ox, oy, ix, iy, sx, sy, a0) in corners {
        let r_o = (r_field as f32).clamp(0.0, max_r);
        let r_i = (r_o - width).max(0.0);
        // Arc centers: corner pulled inward along each axis by the radius.
        let oc = egui::pos2(ox + sx * r_o, oy + sy * r_o);
        let ic = egui::pos2(ix + sx * r_i, iy + sy * r_i);
        let segs = if r_o < 0.5 {
            0
        } else {
            (r_o * 0.5).ceil().clamp(1.0, 16.0) as usize
        };
        for k in 0..=segs {
            let a = a0 + FRAC_PI_2 * (k as f32 / segs.max(1) as f32);
            let dir = egui::vec2(a.cos(), a.sin());
            outer_pts.push(oc + r_o * dir);
            inner_pts.push(ic + r_i * dir);
        }
    }

    let n = outer_pts.len() as u32;
    let mut mesh = Mesh::default();
    for p in &outer_pts {
        mesh.colored_vertex(*p, outer_color(*p));
    }
    for p in &inner_pts {
        mesh.colored_vertex(*p, inner_color(*p));
    }
    for k in 0..n {
        let next = (k + 1) % n;
        let (oa, ob) = (k, next);
        let (ia, ib) = (n + k, n + next);
        mesh.add_triangle(oa, ob, ib);
        mesh.add_triangle(oa, ib, ia);
    }
    mesh
}

/// Add a straight glow band along one edge of `rect` to `mesh`: full `color` at
/// the outer edge fading to transparent `w` pixels inward.
fn push_glow_band(mesh: &mut Mesh, rect: Rect, side: Sides, w: f32, color: Color32) {
    // (outer_a, outer_b, inner_a, inner_b) — outer two carry `color`, inner two transparent.
    let (oa, ob, ia, ib) = if side == Sides::TOP {
        (
            rect.left_top(),
            rect.right_top(),
            egui::pos2(rect.left(), rect.top() + w),
            egui::pos2(rect.right(), rect.top() + w),
        )
    } else if side == Sides::BOTTOM {
        (
            rect.left_bottom(),
            rect.right_bottom(),
            egui::pos2(rect.left(), rect.bottom() - w),
            egui::pos2(rect.right(), rect.bottom() - w),
        )
    } else if side == Sides::LEFT {
        (
            rect.left_top(),
            rect.left_bottom(),
            egui::pos2(rect.left() + w, rect.top()),
            egui::pos2(rect.left() + w, rect.bottom()),
        )
    } else {
        // RIGHT
        (
            rect.right_top(),
            rect.right_bottom(),
            egui::pos2(rect.right() - w, rect.top()),
            egui::pos2(rect.right() - w, rect.bottom()),
        )
    };
    let base = mesh.vertices.len() as u32;
    mesh.colored_vertex(oa, color);
    mesh.colored_vertex(ob, color);
    mesh.colored_vertex(ia, Color32::TRANSPARENT);
    mesh.colored_vertex(ib, Color32::TRANSPARENT);
    mesh.add_triangle(base, base + 1, base + 3);
    mesh.add_triangle(base, base + 3, base + 2);
}

/// Build a smooth inner-glow shape: full `color` at the rect edge fading to
/// transparent `width` pixels inward.
///
/// When `glow.sides` is [`Sides::ALL`], the glow is a full ring built with a
/// rounded ring mesh that follows the corner radius `cr`. For any partial
/// side selection the glow is drawn as straight bands from the chosen edges
/// (corners are not rounded; overlapping bands at a shared corner blend
/// additively). The fade is GPU-interpolated in premultiplied alpha, so there
/// is no banding either way.
///
/// Returns `None` when `glow.width < 0.5`, the color is fully transparent, or no
/// side is selected.
pub fn inner_glow_shape(rect: Rect, cr: CornerRadius, glow: InnerGlow) -> Option<Shape> {
    let max_w = (rect.width().min(rect.height()) / 2.0).max(0.0);
    let w = glow.width.clamp(0.0, max_w);
    if w < 0.5 || glow.color.a() == 0 || !glow.sides.any() {
        return None;
    }
    let edge = glow.color;
    if glow.sides.is_all() {
        let mesh = rounded_ring_mesh(rect, cr, w, |_| edge, |_| Color32::TRANSPARENT);
        return Some(Shape::Mesh(mesh.into()));
    }
    let mut mesh = Mesh::default();
    if glow.sides.top {
        push_glow_band(&mut mesh, rect, Sides::TOP, w, edge);
    }
    if glow.sides.bottom {
        push_glow_band(&mut mesh, rect, Sides::BOTTOM, w, edge);
    }
    if glow.sides.left {
        push_glow_band(&mut mesh, rect, Sides::LEFT, w, edge);
    }
    if glow.sides.right {
        push_glow_band(&mut mesh, rect, Sides::RIGHT, w, edge);
    }
    Some(Shape::Mesh(mesh.into()))
}

/// Build the 4-trapezoid mitered mesh for a vertically-interpolated gradient border.
///
/// The border sits **inside** `rect` so the widget's layout rect is unchanged.
/// Corner-radius is not respected (straight mitered edges — same accepted limitation
/// as the existing per-side line-segment borders).
pub fn border_gradient_mesh(rect: Rect, g: BorderGradient) -> Mesh {
    let w = g
        .width
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let inner = rect.shrink(w);

    // Color at a given y coordinate (top color at the top edge, bottom at the bottom).
    let col = move |p: egui::Pos2| {
        let t = ((p.y - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
        g.top.lerp_to_gamma(g.bottom, t)
    };

    ring_mesh(rect, inner, col, col)
}

/// Paint the gradient underlay (solid bg + gradient) into a pre-reserved slot.
///
/// When `bg_gradient` is set for the resolved state, `apply_to_visuals` has
/// already suppressed the widget's own bg fill by setting it to TRANSPARENT.
/// This function repaints the solid `bg` and the gradient on top of it.
///
/// Call pattern: reserve a `Shape::Noop` slot _before_ the widget renders, then
/// call this _after_ the response is available.
pub fn paint_widget_gradient_underlay(
    ui: &egui::Ui,
    slot: egui::layers::ShapeIdx,
    rect: Rect,
    cr: CornerRadius,
    state: &ResolvedStyle,
) {
    if let Some(g) = &state.bg_gradient {
        let shapes = vec![
            Shape::rect_filled(rect, cr, state.bg),
            gradient_shape(ui.ctx(), rect, cr, g),
        ];
        ui.painter().set(slot, Shape::Vec(shapes));
    }
}

/// Paint border-gradient and/or inner-glow overlays after the widget renders.
///
/// `border_gradient` wins over per-side overrides and the uniform border; both
/// can coexist with `inner_glow` (glow paints last, on top).
///
/// Call this in place of [`SharedStyle::paint_widget_side_borders`].
pub fn paint_widget_overlays(ui: &egui::Ui, rect: Rect, state: &ResolvedStyle) {
    if let Some(bg) = state.border_gradient {
        ui.painter()
            .add(Shape::Mesh(border_gradient_mesh(rect, bg).into()));
    } else if state.has_border_overrides {
        paint_side_borders(ui.painter(), rect, state.border_sides);
    }
    if let Some(glow) = state.inner_glow
        && let Some(shape) = inner_glow_shape(rect, state.corner_radius, glow)
    {
        ui.painter().add(shape);
    }
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
///
/// # Precedence rules
///
/// Three independent groups of fields resolve in a fixed order when more than
/// one is set at once. This is the single place that lists all three; each
/// implementing site links back here rather than restating it.
///
/// **Pseudo-state colors** (`bg`, `border`, `text_color`, `accent`, and their
/// gradient / glow counterparts) - resolved in [`resolve`](Self::resolve),
/// per-field, against whichever of `hovered` / `active` / `focused` the
/// current [`PseudoState`] has set:
/// - `bg` and the gradient/glow fields: `active` > `hover` > `focus` > base.
/// - `border` (and per-side border overrides, which each fall back to the
///   resolved uniform `border` when unset for that state): `focus` > `hover` > base.
///   There is no `active_border` - a widget's border doesn't change on press,
///   only on hover/focus.
/// - `text_color`: `hover` > base only. There is no `active_text_color` or
///   `focus_text_color` yet.
/// - A pseudo-state's own field always wins over falling through to the base
///   field, which itself falls through to egui's active `Visuals` when unset.
///
/// **Border decorations** (`border`, the per-side `border_sides` overrides,
/// and `border_gradient`), for a single resolved state - see
/// [`StyledFrame::show`](crate::StyledFrame) for the paint-order
/// implementation: `border_gradient` > per-side overrides > uniform `border`.
/// Only one of the three ever paints; they are not layered. Per-side and
/// gradient borders both draw straight edges and do not follow
/// `corner_radius` - only the uniform `border` path does (delegated to
/// egui's own rounded stroke).
///
/// **Sizing** (`width_pct` / `height_pct`, `aspect_ratio`, `full_width` /
/// `full_height`, `min_width` / `max_width` / `min_height` / `max_height`,
/// and `fill_size` on frame-backed containers) - see
/// [`resolve_size`](Self::resolve_size) for the exact width/height chains
/// and the non-finite-`avail` degradation. Short version: an explicit
/// percentage or `fill_size` wins outright; `aspect_ratio` only kicks in once
/// a definite width exists; `full_width`/`full_height` is the fallback that
/// stretches to available space; `min_*`/`max_*` clamp whichever of the above
/// was chosen, with `min_*` applied last so an explicit minimum always wins.
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
    /// Width-to-height ratio (CSS convention: `16.0/9.0` is a wide box).
    /// Derives height from a resolved definite width (`width_pct` or `full_width`).
    /// No-op when no definite width is available. Overridden by `height_pct`/`full_height`.
    pub aspect_ratio: Option<f32>,

    // Cursor
    pub cursor_icon: Option<CursorIcon>,

    // Visibility — `Some(false)` calls `Ui::set_invisible()`: reserves layout
    // space, skips painting, implies disabled (no hover/click interaction).
    // `None` and `Some(true)` are both fully visible.
    pub visible: Option<bool>,

    // Gradient background (paints over solid `bg`, like CSS background-image over background-color)
    pub bg_gradient: Option<Gradient>,
    pub hover_bg_gradient: Option<Gradient>,
    pub active_bg_gradient: Option<Gradient>,
    pub focus_bg_gradient: Option<Gradient>,

    // Inner glow (inward fade from the rect edge)
    pub inner_glow: Option<InnerGlow>,
    pub hover_inner_glow: Option<InnerGlow>,
    pub active_inner_glow: Option<InnerGlow>,
    pub focus_inner_glow: Option<InnerGlow>,

    // Gradient border (vertical color interpolation; wins over uniform border and per-side overrides)
    pub border_gradient: Option<BorderGradient>,
    pub hover_border_gradient: Option<BorderGradient>,
    pub active_border_gradient: Option<BorderGradient>,
    pub focus_border_gradient: Option<BorderGradient>,

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

/// Output of [`SharedStyle::resolve_size`] — pre-computed dimensions for all
/// sizing properties. Pass to [`ResolvedSize::apply_to_ui`] or inspect
/// individual fields for widget-specific sizing APIs (e.g. `cb.width()`).
pub struct ResolvedSize {
    /// Definite width — both min and max should be pinned to this value.
    /// Produced by `width_pct` or `full_width` (capped by `max_width`).
    pub definite_w: Option<f32>,
    /// Definite height — both min and max should be pinned to this value.
    /// Produced by `height_pct`, `aspect_ratio`, or `full_height`.
    pub definite_h: Option<f32>,
    /// Pass-through minimum width (only set when `definite_w` is `None`).
    pub min_w: Option<f32>,
    /// Pass-through maximum width (only set when `definite_w` is `None`).
    pub max_w: Option<f32>,
    /// Pass-through minimum height (only set when `definite_h` is `None`).
    pub min_h: Option<f32>,
    /// Pass-through maximum height (only set when `definite_h` is `None`).
    pub max_h: Option<f32>,
}

impl ResolvedSize {
    /// Apply all sizing constraints to a `Ui`. Definite values pin both min
    /// and max; otherwise max is applied first so `full_width` reads the
    /// capped available size, then min.
    pub fn apply_to_ui(&self, ui: &mut egui::Ui) {
        if let Some(w) = self.definite_w {
            ui.set_min_width(w);
            ui.set_max_width(w);
        } else {
            if let Some(w) = self.max_w {
                ui.set_max_width(w);
            }
            if let Some(w) = self.min_w {
                ui.set_min_width(w);
            }
        }
        if let Some(h) = self.definite_h {
            ui.set_min_height(h);
            ui.set_max_height(h);
        } else {
            if let Some(h) = self.max_h {
                ui.set_max_height(h);
            }
            if let Some(h) = self.min_h {
                ui.set_min_height(h);
            }
        }
    }
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
    pub bg_gradient: Option<Gradient>,
    pub inner_glow: Option<InnerGlow>,
    pub border_gradient: Option<BorderGradient>,
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

    /// Derive height from `aspect_ratio` and a known definite width.
    /// Returns `None` when unset or `ratio <= 0`. Clamped by `min_height`/`max_height`.
    pub fn resolved_aspect_height(&self, width: f32) -> Option<f32> {
        self.aspect_ratio.filter(|&r| r > 0.0).map(|r| {
            let mut h = width / r;
            if let Some(m) = self.max_height {
                h = h.min(m);
            }
            if let Some(m) = self.min_height {
                h = h.max(m);
            }
            h
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

    /// Centralized size resolver. See the "Precedence rules" section on
    /// [`SharedStyle`] for how this fits with the other two precedence
    /// chains (colors, border decorations). Encodes the full sizing
    /// precedence for all styled widgets:
    /// - Width: `width_pct` > `full_width` (capped by `max_width`) > pass-through `min/max_width`
    /// - Height: `height_pct` > `aspect_ratio` (from definite width) > `full_height` > pass-through
    ///
    /// Non-finite `avail` values (floating [`StyledArea`](crate::StyledArea) without `fill_screen`)
    /// are clamped to `0.0`, so `full_width`/`width_pct` degrade to content-sized
    /// rather than setting an infinite minimum width.
    pub fn resolve_size(&self, avail_w: f32, avail_h: f32) -> ResolvedSize {
        let avail_w = if avail_w.is_finite() { avail_w } else { 0.0 };
        let avail_h = if avail_h.is_finite() { avail_h } else { 0.0 };

        let definite_w = if let Some(w) = self.resolved_width_pct(avail_w) {
            Some(w)
        } else if self.full_width && avail_w > 0.0 {
            let w = self.max_width.map_or(avail_w, |m| avail_w.min(m));
            Some(self.min_width.map_or(w, |m| w.max(m)))
        } else {
            None
        };

        let definite_h = if let Some(h) = self.resolved_height_pct(avail_h) {
            Some(h)
        } else if let Some(dw) = definite_w {
            self.resolved_aspect_height(dw)
        } else if self.full_height && avail_h > 0.0 {
            let h = self.max_height.map_or(avail_h, |m| avail_h.min(m));
            Some(self.min_height.map_or(h, |m| h.max(m)))
        } else {
            None
        };

        ResolvedSize {
            definite_w,
            definite_h,
            min_w: if definite_w.is_none() {
                self.min_width
            } else {
                None
            },
            max_w: if definite_w.is_none() {
                self.max_width
            } else {
                None
            },
            min_h: if definite_h.is_none() {
                self.min_height
            } else {
                None
            },
            max_h: if definite_h.is_none() {
                self.max_height
            } else {
                None
            },
        }
    }

    /// Resolve against current pseudo-state and egui's active visuals. See
    /// the "Precedence rules" section on [`SharedStyle`] for the per-field
    /// state precedence this implements.
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

        let bg_gradient = match state {
            _ if state.active && self.active_bg_gradient.is_some() => {
                self.active_bg_gradient.clone()
            }
            _ if state.hovered && self.hover_bg_gradient.is_some() => {
                self.hover_bg_gradient.clone()
            }
            _ if state.focused && self.focus_bg_gradient.is_some() => {
                self.focus_bg_gradient.clone()
            }
            _ => self.bg_gradient.clone(),
        };
        let inner_glow = match state {
            _ if state.active && self.active_inner_glow.is_some() => self.active_inner_glow,
            _ if state.hovered && self.hover_inner_glow.is_some() => self.hover_inner_glow,
            _ if state.focused && self.focus_inner_glow.is_some() => self.focus_inner_glow,
            _ => self.inner_glow,
        };
        let border_gradient = match state {
            _ if state.active && self.active_border_gradient.is_some() => {
                self.active_border_gradient
            }
            _ if state.hovered && self.hover_border_gradient.is_some() => {
                self.hover_border_gradient
            }
            _ if state.focused && self.focus_border_gradient.is_some() => {
                self.focus_border_gradient
            }
            _ => self.border_gradient,
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
            bg_gradient,
            inner_glow,
            border_gradient,
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
            // When a gradient is set we paint bg ourselves (underlay slot), so
            // suppress egui's own fill to avoid a solid-color flash underneath.
            wv.bg_fill = if resolved.bg_gradient.is_some() {
                Color32::TRANSPARENT
            } else {
                resolved.bg
            };
            wv.weak_bg_fill = wv.bg_fill;
            // Suppress egui's uniform stroke when we paint borders ourselves.
            wv.bg_stroke = if resolved.border_gradient.is_some() || resolved.has_border_overrides {
                Stroke::NONE
            } else {
                resolved.border
            };
            wv.corner_radius = per.corner_radius;
            wv.expansion = 0.0;
            wv.fg_stroke = Stroke::new(wv.fg_stroke.width, resolved.text_color);
        }
        // Also update the open state for combo-box menus.
        let open_bg = if per.inactive.bg_gradient.is_some() {
            Color32::TRANSPARENT
        } else {
            per.inactive.bg
        };
        vis.widgets.open.bg_fill = open_bg;
        vis.widgets.open.weak_bg_fill = open_bg;
        vis.widgets.open.bg_stroke =
            if per.inactive.border_gradient.is_some() || per.inactive.has_border_overrides {
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
            || self.bg_gradient.is_some()
            || self.hover_bg_gradient.is_some()
            || self.active_bg_gradient.is_some()
            || self.focus_bg_gradient.is_some()
            || self.inner_glow.is_some()
            || self.hover_inner_glow.is_some()
            || self.active_inner_glow.is_some()
            || self.focus_inner_glow.is_some()
            || self.border_gradient.is_some()
            || self.hover_border_gradient.is_some()
            || self.active_border_gradient.is_some()
            || self.focus_border_gradient.is_some()
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
        let style = SharedStyle {
            width_pct: Some(50.0),
            ..Default::default()
        };
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
        assert!(
            (w - 120.0).abs() < 1e-4,
            "50% of 400 clamped to max_width 120, got {w}"
        );
    }

    #[test]
    fn resolved_width_pct_floored_by_min_width() {
        let style = SharedStyle {
            width_pct: Some(10.0),
            min_width: Some(80.0),
            ..Default::default()
        };
        let w = style.resolved_width_pct(400.0).unwrap();
        assert!(
            (w - 80.0).abs() < 1e-4,
            "10% of 400=40 raised to min_width 80, got {w}"
        );
    }

    #[test]
    fn resolved_height_pct_half_of_avail() {
        let style = SharedStyle {
            height_pct: Some(50.0),
            ..Default::default()
        };
        let h = style.resolved_height_pct(300.0).unwrap();
        assert!((h - 150.0).abs() < 1e-4, "50% of 300 = 150, got {h}");
    }

    // ── distribution_spacing pure-math tests ────────────────────────────────

    #[test]
    fn dist_between_equal_gaps() {
        // avail=400, W=100, n=3 → slack=300, L=0, G=150
        let (l, g) = distribution_spacing(Distribution::SpaceBetween, 300.0, 3, 0.0);
        assert!((l - 0.0).abs() < 1e-4, "between: L should be 0, got {l}");
        assert!(
            (g - 150.0).abs() < 1e-4,
            "between: G should be 150, got {g}"
        );
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
        assert!(
            (l_around - 100.0).abs() < 1e-4,
            "around n=1: L=100, got {l_around}"
        );
        assert!(
            (l_evenly - 100.0).abs() < 1e-4,
            "evenly n=1: L=100, got {l_evenly}"
        );
    }

    #[test]
    fn dist_zero_slack_uses_min_gap() {
        let (l, g) = distribution_spacing(Distribution::SpaceBetween, 0.0, 3, 8.0);
        assert!((l - 0.0).abs() < 1e-4, "zero slack: L=0, got {l}");
        assert!((g - 8.0).abs() < 1e-4, "zero slack: G=min_gap=8, got {g}");
    }

    #[test]
    fn resolved_aspect_height_none_when_unset() {
        let s = SharedStyle::default();
        assert!(s.resolved_aspect_height(200.0).is_none());
    }

    #[test]
    fn resolved_aspect_height_none_when_ratio_zero_or_negative() {
        let s = SharedStyle {
            aspect_ratio: Some(0.0),
            ..Default::default()
        };
        assert!(s.resolved_aspect_height(200.0).is_none());
        let s2 = SharedStyle {
            aspect_ratio: Some(-1.0),
            ..Default::default()
        };
        assert!(s2.resolved_aspect_height(200.0).is_none());
    }

    #[test]
    fn resolved_aspect_height_basic() {
        let s = SharedStyle {
            aspect_ratio: Some(2.0),
            ..Default::default()
        };
        assert!((s.resolved_aspect_height(200.0).unwrap() - 100.0).abs() < 1e-4);
    }

    #[test]
    fn resolved_aspect_height_16_9() {
        let s = SharedStyle {
            aspect_ratio: Some(16.0 / 9.0),
            ..Default::default()
        };
        let h = s.resolved_aspect_height(320.0).unwrap();
        assert!((h - 180.0).abs() < 1e-3, "16:9 of 320 → 180, got {h}");
    }

    #[test]
    fn resolved_aspect_height_clamped_by_max() {
        let s = SharedStyle {
            aspect_ratio: Some(1.0),
            max_height: Some(50.0),
            ..Default::default()
        };
        assert!((s.resolved_aspect_height(200.0).unwrap() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn resolved_aspect_height_floored_by_min() {
        let s = SharedStyle {
            aspect_ratio: Some(10.0),
            min_height: Some(80.0),
            ..Default::default()
        };
        // width/ratio = 200/10 = 20, but min_height floors to 80
        assert!((s.resolved_aspect_height(200.0).unwrap() - 80.0).abs() < 1e-4);
    }

    #[test]
    fn resolve_size_width_pct_50_on_400() {
        let s = SharedStyle {
            width_pct: Some(50.0),
            ..Default::default()
        };
        let sz = s.resolve_size(400.0, 300.0);
        assert!((sz.definite_w.unwrap() - 200.0).abs() < 1e-4);
        assert!(sz.definite_h.is_none());
    }

    #[test]
    fn resolve_size_aspect_from_pct() {
        let s = SharedStyle {
            width_pct: Some(50.0),
            aspect_ratio: Some(2.0),
            ..Default::default()
        };
        let sz = s.resolve_size(400.0, 300.0);
        // width = 50% of 400 = 200; height = 200/2 = 100
        assert!((sz.definite_w.unwrap() - 200.0).abs() < 1e-4);
        assert!((sz.definite_h.unwrap() - 100.0).abs() < 1e-4);
    }

    #[test]
    fn resolve_size_non_finite_avail_produces_no_definite() {
        let s = SharedStyle {
            full_width: true,
            ..Default::default()
        };
        let sz = s.resolve_size(f32::INFINITY, f32::INFINITY);
        assert!(sz.definite_w.is_none());
        assert!(sz.definite_h.is_none());
    }

    // ── BgGradient / InnerGlow / BorderGradient resolution tests ─────────────

    fn style_with_all_gradients() -> SharedStyle {
        SharedStyle {
            bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                Color32::RED,
                Color32::BLUE,
            ))),
            hover_bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                Color32::GREEN,
                Color32::BLUE,
            ))),
            active_bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                Color32::YELLOW,
                Color32::BLUE,
            ))),
            focus_bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                Color32::WHITE,
                Color32::BLUE,
            ))),
            ..Default::default()
        }
    }

    #[test]
    fn bg_gradient_active_beats_hovered_and_focused() {
        let style = style_with_all_gradients();
        let resolved = style.resolve(
            PseudoState {
                hovered: true,
                active: true,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.bg_gradient,
            Some(Gradient::Corners(BgGradient::vertical(
                Color32::YELLOW,
                Color32::BLUE
            )))
        );
    }

    #[test]
    fn bg_gradient_hover_beats_focused() {
        let style = style_with_all_gradients();
        let resolved = style.resolve(
            PseudoState {
                hovered: true,
                active: false,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.bg_gradient,
            Some(Gradient::Corners(BgGradient::vertical(
                Color32::GREEN,
                Color32::BLUE
            )))
        );
    }

    #[test]
    fn bg_gradient_focus_only() {
        let style = style_with_all_gradients();
        let resolved = style.resolve(
            PseudoState {
                hovered: false,
                active: false,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.bg_gradient,
            Some(Gradient::Corners(BgGradient::vertical(
                Color32::WHITE,
                Color32::BLUE
            )))
        );
    }

    #[test]
    fn bg_gradient_falls_back_to_base() {
        let style = style_with_all_gradients();
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert_eq!(
            resolved.bg_gradient,
            Some(Gradient::Corners(BgGradient::vertical(
                Color32::RED,
                Color32::BLUE
            )))
        );
    }

    fn style_with_all_glows() -> SharedStyle {
        SharedStyle {
            inner_glow: Some(InnerGlow::new(4.0, Color32::RED)),
            hover_inner_glow: Some(InnerGlow::new(8.0, Color32::GREEN)),
            active_inner_glow: Some(InnerGlow::new(12.0, Color32::YELLOW)),
            focus_inner_glow: Some(InnerGlow::new(6.0, Color32::WHITE)),
            ..Default::default()
        }
    }

    #[test]
    fn inner_glow_active_beats_hovered_and_focused() {
        let style = style_with_all_glows();
        let resolved = style.resolve(
            PseudoState {
                hovered: true,
                active: true,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.inner_glow,
            Some(InnerGlow::new(12.0, Color32::YELLOW))
        );
    }

    #[test]
    fn inner_glow_hover_beats_focused() {
        let style = style_with_all_glows();
        let resolved = style.resolve(
            PseudoState {
                hovered: true,
                active: false,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.inner_glow,
            Some(InnerGlow::new(8.0, Color32::GREEN))
        );
    }

    #[test]
    fn inner_glow_falls_back_to_base() {
        let style = style_with_all_glows();
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert_eq!(resolved.inner_glow, Some(InnerGlow::new(4.0, Color32::RED)));
    }

    fn style_with_all_border_gradients() -> SharedStyle {
        SharedStyle {
            border_gradient: Some(BorderGradient::new(2.0, Color32::RED, Color32::BLUE)),
            hover_border_gradient: Some(BorderGradient::new(3.0, Color32::GREEN, Color32::BLUE)),
            active_border_gradient: Some(BorderGradient::new(4.0, Color32::YELLOW, Color32::BLUE)),
            focus_border_gradient: Some(BorderGradient::new(2.5, Color32::WHITE, Color32::BLUE)),
            ..Default::default()
        }
    }

    #[test]
    fn border_gradient_active_beats_hovered_and_focused() {
        let style = style_with_all_border_gradients();
        let resolved = style.resolve(
            PseudoState {
                hovered: true,
                active: true,
                focused: true,
            },
            &default_visuals(),
        );
        assert_eq!(
            resolved.border_gradient,
            Some(BorderGradient::new(4.0, Color32::YELLOW, Color32::BLUE))
        );
    }

    #[test]
    fn border_gradient_falls_back_to_base() {
        let style = style_with_all_border_gradients();
        let resolved = style.resolve(PseudoState::default(), &default_visuals());
        assert_eq!(
            resolved.border_gradient,
            Some(BorderGradient::new(2.0, Color32::RED, Color32::BLUE))
        );
    }

    #[test]
    fn has_frame_styles_triggers_for_gradient_fields() {
        for style in [
            SharedStyle {
                bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                    Color32::RED,
                    Color32::BLUE,
                ))),
                ..Default::default()
            },
            SharedStyle {
                hover_bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                    Color32::RED,
                    Color32::BLUE,
                ))),
                ..Default::default()
            },
            SharedStyle {
                inner_glow: Some(InnerGlow::new(4.0, Color32::RED)),
                ..Default::default()
            },
            SharedStyle {
                hover_inner_glow: Some(InnerGlow::new(4.0, Color32::RED)),
                ..Default::default()
            },
            SharedStyle {
                border_gradient: Some(BorderGradient::new(2.0, Color32::RED, Color32::BLUE)),
                ..Default::default()
            },
            SharedStyle {
                hover_border_gradient: Some(BorderGradient::new(2.0, Color32::RED, Color32::BLUE)),
                ..Default::default()
            },
        ] {
            assert!(style.has_frame_styles());
        }
    }

    #[test]
    fn apply_to_visuals_suppresses_fill_when_bg_gradient() {
        use egui::Visuals;
        let ctx = egui::Context::default();
        let visuals = Visuals::default();
        let style = SharedStyle {
            bg_gradient: Some(Gradient::Corners(BgGradient::vertical(
                Color32::RED,
                Color32::BLUE,
            ))),
            ..Default::default()
        };
        let per = style.resolve_per_state(&visuals);
        let mut vis = visuals.clone();
        SharedStyle::apply_to_visuals(&per, &mut vis);
        assert_eq!(vis.widgets.inactive.bg_fill, Color32::TRANSPARENT);
        assert_eq!(vis.widgets.inactive.weak_bg_fill, Color32::TRANSPARENT);
        let _ = ctx;
    }

    #[test]
    fn apply_to_visuals_suppresses_stroke_when_border_gradient() {
        use egui::Visuals;
        let visuals = Visuals::default();
        let style = SharedStyle {
            border_gradient: Some(BorderGradient::new(2.0, Color32::RED, Color32::BLUE)),
            ..Default::default()
        };
        let per = style.resolve_per_state(&visuals);
        let mut vis = visuals.clone();
        SharedStyle::apply_to_visuals(&per, &mut vis);
        assert_eq!(vis.widgets.inactive.bg_stroke, Stroke::NONE);
    }

    #[test]
    fn bg_gradient_constructors() {
        let v = BgGradient::vertical(Color32::RED, Color32::BLUE);
        assert_eq!(v.top_left, Color32::RED);
        assert_eq!(v.top_right, Color32::RED);
        assert_eq!(v.bottom_left, Color32::BLUE);
        assert_eq!(v.bottom_right, Color32::BLUE);

        let h = BgGradient::horizontal(Color32::RED, Color32::BLUE);
        assert_eq!(h.top_left, Color32::RED);
        assert_eq!(h.top_right, Color32::BLUE);
        assert_eq!(h.bottom_left, Color32::RED);
        assert_eq!(h.bottom_right, Color32::BLUE);
    }

    #[test]
    fn gradient_texture_cached_by_colors() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        let g = BgGradient::vertical(Color32::RED, Color32::BLUE);
        let h1 = gradient_texture(&ctx, g);
        let h2 = gradient_texture(&ctx, g);
        assert_eq!(h1.id(), h2.id());

        let g2 = BgGradient::vertical(Color32::GREEN, Color32::BLUE);
        let h3 = gradient_texture(&ctx, g2);
        assert_ne!(h1.id(), h3.id());
    }

    #[test]
    fn bg_gradient_shape_uses_texel_center_uv() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        let g = BgGradient::vertical(Color32::RED, Color32::BLUE);
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let shape = bg_gradient_shape(&ctx, rect, CornerRadius::default(), g);
        match shape {
            Shape::Rect(rs) => {
                let brush = rs.brush.expect("gradient shape must have a brush");
                let expected = egui::Rect::from_min_max(pos2(0.25, 0.25), pos2(0.75, 0.75));
                assert!((brush.uv.min.x - expected.min.x).abs() < 1e-5);
                assert!((brush.uv.min.y - expected.min.y).abs() < 1e-5);
                assert!((brush.uv.max.x - expected.max.x).abs() < 1e-5);
                assert!((brush.uv.max.y - expected.max.y).abs() < 1e-5);
            }
            other => panic!("expected Shape::Rect, got {:?}", other),
        }
    }

    #[test]
    fn linear_gradient_sorts_and_samples_stops() {
        // Pass stops out of order; they should be sorted.
        let g = LinearGradient::new(
            [(1.0, Color32::BLUE), (0.0, Color32::RED)],
            GradientAxis::Vertical,
        );
        assert_eq!(g.stops[0].1, Color32::RED);
        assert_eq!(g.stops[1].1, Color32::BLUE);
        // Endpoints exact, midpoint between the two.
        assert_eq!(g.sample(0.0), Color32::RED);
        assert_eq!(g.sample(1.0), Color32::BLUE);
        let mid = g.sample(0.5);
        assert!(mid != Color32::RED && mid != Color32::BLUE);
    }

    #[test]
    fn linear_gradient_clamps_outside_range() {
        let g = LinearGradient::new(
            [(0.25, Color32::RED), (0.75, Color32::BLUE)],
            GradientAxis::Vertical,
        );
        assert_eq!(g.sample(0.0), Color32::RED); // below first stop
        assert_eq!(g.sample(1.0), Color32::BLUE); // above last stop
    }

    #[test]
    fn linear_gradient_evenly_spaced_positions() {
        let g = LinearGradient::evenly_spaced(
            [Color32::RED, Color32::GREEN, Color32::BLUE],
            GradientAxis::Horizontal,
        );
        assert_eq!(g.stops.len(), 3);
        assert!((g.stops[0].0 - 0.0).abs() < 1e-6);
        assert!((g.stops[1].0 - 0.5).abs() < 1e-6);
        assert!((g.stops[2].0 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn linear_gradient_texture_cached_and_axis_sized() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        let v = LinearGradient::new(
            [(0.0, Color32::RED), (1.0, Color32::BLUE)],
            GradientAxis::Vertical,
        );
        let h1 = linear_gradient_texture(&ctx, &v);
        let h2 = linear_gradient_texture(&ctx, &v);
        assert_eq!(h1.id(), h2.id());
        // A horizontal gradient of the same stops is a different texture.
        let h = LinearGradient::new(
            [(0.0, Color32::RED), (1.0, Color32::BLUE)],
            GradientAxis::Horizontal,
        );
        let h3 = linear_gradient_texture(&ctx, &h);
        assert_ne!(h1.id(), h3.id());
    }

    #[test]
    fn linear_gradient_shape_axis_uv() {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput::default());
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let inset = 0.5 / LINEAR_RAMP_TEXELS as f32;

        let v = LinearGradient::new(
            [(0.0, Color32::RED), (1.0, Color32::BLUE)],
            GradientAxis::Vertical,
        );
        if let Shape::Rect(rs) = linear_gradient_shape(&ctx, rect, CornerRadius::default(), &v) {
            let uv = rs.brush.unwrap().uv;
            // Vertical: x spans full 0..1, y inset by half a texel.
            assert!((uv.min.x - 0.0).abs() < 1e-6 && (uv.max.x - 1.0).abs() < 1e-6);
            assert!((uv.min.y - inset).abs() < 1e-6 && (uv.max.y - (1.0 - inset)).abs() < 1e-6);
        } else {
            panic!("expected Shape::Rect");
        }
    }

    #[test]
    fn inner_glow_per_side_emits_only_selected_bands() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let glow = InnerGlow::with_sides(8.0, Color32::WHITE, Sides::Y);
        let shape = inner_glow_shape(rect, CornerRadius::default(), glow).unwrap();
        if let Shape::Mesh(mesh) = shape {
            // Two bands (top + bottom), 4 verts each = 8 vertices, 12 indices.
            assert_eq!(mesh.vertices.len(), 8);
            assert_eq!(mesh.indices.len(), 12);
            // Every vertex lies on the top or bottom band (y at 0, 8, 92, or 100).
            for v in &mesh.vertices {
                let y = v.pos.y;
                let on_band = y < 1e-3
                    || (y - 8.0).abs() < 1e-3
                    || (y - 92.0).abs() < 1e-3
                    || (y - 100.0).abs() < 1e-3;
                assert!(on_band, "vertex y={y} not on a top/bottom band");
            }
        } else {
            panic!("expected Shape::Mesh");
        }
    }

    #[test]
    fn inner_glow_no_sides_emits_nothing() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let none = Sides {
            top: false,
            right: false,
            bottom: false,
            left: false,
        };
        let glow = InnerGlow::with_sides(8.0, Color32::WHITE, none);
        assert!(inner_glow_shape(rect, CornerRadius::default(), glow).is_none());
    }

    #[test]
    fn inner_glow_shape_full_at_edge_transparent_inward() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let glow = InnerGlow::new(16.0, Color32::from_rgba_premultiplied(255, 255, 255, 200));
        // Sharp corners (radius 0): 4 corner points → 4 outer + 4 inner = 8 vertices.
        let shape = inner_glow_shape(rect, CornerRadius::default(), glow)
            .expect("glow should emit a shape");
        match shape {
            Shape::Mesh(mesh) => {
                assert!(mesh.vertices.len() >= 8);
                assert_eq!(mesh.vertices.len() % 2, 0);
                let half = mesh.vertices.len() / 2;
                // Outer half carries the glow color; inner half is transparent.
                for v in &mesh.vertices[0..half] {
                    assert_eq!(v.color, glow.color);
                }
                for v in &mesh.vertices[half..] {
                    assert_eq!(v.color, Color32::TRANSPARENT);
                }
            }
            other => panic!("expected Shape::Mesh, got {:?}", other),
        }
    }

    #[test]
    fn inner_glow_rounded_corners_add_vertices() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let glow = InnerGlow::new(8.0, Color32::WHITE);
        let sharp = inner_glow_shape(rect, CornerRadius::default(), glow).unwrap();
        let rounded = inner_glow_shape(rect, CornerRadius::same(20), glow).unwrap();
        let (sharp_n, rounded_n) = match (sharp, rounded) {
            (Shape::Mesh(a), Shape::Mesh(b)) => (a.vertices.len(), b.vertices.len()),
            _ => panic!("expected meshes"),
        };
        // Rounded corners tessellate the arcs, producing strictly more vertices.
        assert!(rounded_n > sharp_n);
    }

    #[test]
    fn inner_glow_zero_width_emits_nothing() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        assert!(
            inner_glow_shape(
                rect,
                CornerRadius::default(),
                InnerGlow::new(0.0, Color32::RED)
            )
            .is_none()
        );
    }

    #[test]
    fn inner_glow_transparent_color_emits_nothing() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        assert!(
            inner_glow_shape(
                rect,
                CornerRadius::default(),
                InnerGlow::new(8.0, Color32::TRANSPARENT)
            )
            .is_none()
        );
    }

    #[test]
    fn inner_glow_width_clamped_to_half_rect() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(50.0, 30.0));
        let glow = InnerGlow::new(1000.0, Color32::WHITE);
        let shape = inner_glow_shape(rect, CornerRadius::default(), glow)
            .expect("glow should emit a shape");
        // Inner ring is clamped to half the min dimension (15px), so its vertices
        // collapse toward the rect center but never invert past it.
        if let Shape::Mesh(mesh) = shape {
            for v in &mesh.vertices {
                assert!(rect.contains(v.pos) || rect.distance_to_pos(v.pos) < 1e-3);
            }
        } else {
            panic!("expected Shape::Mesh");
        }
    }

    #[test]
    fn border_gradient_mesh_geometry() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0));
        let g = BorderGradient::new(10.0, Color32::RED, Color32::BLUE);
        let mesh = border_gradient_mesh(rect, g);
        assert_eq!(mesh.vertices.len(), 16);
        assert_eq!(mesh.indices.len(), 24);
        // Top-edge outer vertices should have RED color.
        assert_eq!(mesh.vertices[0].color, Color32::RED);
        assert_eq!(mesh.vertices[1].color, Color32::RED);
        // Bottom-edge outer vertices should have BLUE color.
        assert_eq!(mesh.vertices[4].color, Color32::BLUE);
        assert_eq!(mesh.vertices[5].color, Color32::BLUE);
    }

    #[test]
    fn border_gradient_mesh_width_clamped() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0));
        let g = BorderGradient::new(1000.0, Color32::RED, Color32::BLUE);
        let mesh = border_gradient_mesh(rect, g);
        // Inner rect should not go negative — vertices stay inside rect.
        for v in &mesh.vertices {
            assert!(rect.contains(v.pos) || rect.distance_to_pos(v.pos) < 1e-3);
        }
    }
}
