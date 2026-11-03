//! End-of-pass collection of the shapes under each blurred container.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use egui::emath::TSTransform;
use egui::epaint::{ClippedPrimitive, ClippedShape, Mesh, Primitive, mutex::RwLock};
use egui::layers::ShapeIdx;
use egui::{
    Color32, Context, Id, LayerId, Pos2, Rect, Shape, TextureFilter, TextureId, Ui, ViewportId,
};
use egui_wgpu::{Renderer, wgpu};

/// Kawase taps reach well past the nominal radius.
const CULL_FACTOR: f32 = 4.0;
/// Free a container's cached state after this many frames unseen.
pub(crate) const EVICT_AFTER_FRAMES: u64 = 120;
/// Stands in for the scene texture; never handed out by egui_wgpu, which counts up from 0.
const SCENE_TEXTURE: TextureId = TextureId::User(u64::MAX);

/// What the end-of-pass collector hands to the paint callback.
#[derive(Default)]
pub(crate) struct Payload {
    pub primitives: Arc<Vec<ClippedPrimitive>>,
    pub textures: Vec<(TextureId, wgpu::Texture, wgpu::FilterMode)>,
    /// Container rect and backdrop region in global points. `None` if nothing was collected.
    pub rects: Option<(Rect, Rect)>,
    /// False when the backdrop matches last frame's, so last frame's blur can be reused.
    pub changed: bool,
}

#[derive(Clone)]
pub(crate) struct Request {
    pub key: Id,
    pub viewport: ViewportId,
    pub layer: LayerId,
    /// The container's callback shape. Everything before it in `layer` is backdrop.
    pub slot: ShapeIdx,
    pub blur: f32,
    pub payload: Arc<Mutex<Payload>>,
}

fn pending_id() -> Id {
    Id::unique("egui_styled::backdrop::pending")
}

pub(crate) fn push(ctx: &Context, request: Request) {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<Request>>(pending_id())
            .push(request);
    });
}

/// Requests for `viewport`. Others stay queued, since immediate viewports nest inside a pass.
fn take_pending(ctx: &Context, viewport: ViewportId) -> Vec<Request> {
    ctx.data_mut(|d| {
        let all = d.get_temp_mut_or_default::<Vec<Request>>(pending_id());
        let (mine, rest) = std::mem::take(all)
            .into_iter()
            .partition(|r| r.viewport == viewport);
        *all = rest;
        mine
    })
}

/// Last frame's backdrop for one container.
struct Cached {
    ppp: f32,
    rect: Rect,
    region: Rect,
    shapes: Vec<ClippedShape>,
    primitives: Arc<Vec<ClippedPrimitive>>,
    frame: u64,
}

impl Cached {
    fn matches(&self, ppp: f32, rect: Rect, region: Rect, shapes: &[ClippedShape]) -> bool {
        // An image updated in place keeps its id, so only font-atlas backdrops are reusable.
        self.ppp == ppp
            && self.rect == rect
            && self.region == region
            && shapes.iter().all(|s| font_only(&s.shape))
            && self.shapes == shapes
    }
}

fn font_only(shape: &Shape) -> bool {
    match shape {
        Shape::Vec(shapes) => shapes.iter().all(font_only),
        shape => shape.texture_id() == TextureId::default(),
    }
}

pub(crate) struct Collector {
    renderer: Arc<RwLock<Renderer>>,
    cache: HashMap<Id, Cached>,
    /// Textures egui_wgpu holds only as bind groups, so the private renderer can sample them.
    pub native: HashMap<TextureId, wgpu::Texture>,
    /// Drawn under every backdrop, stretched over the viewport.
    pub scene: Option<wgpu::Texture>,
}

impl Collector {
    pub(crate) fn new(renderer: Arc<RwLock<Renderer>>) -> Self {
        Self {
            renderer,
            cache: HashMap::new(),
            native: HashMap::new(),
            scene: None,
        }
    }
}

impl egui::Plugin for Collector {
    fn debug_name(&self) -> &'static str {
        "egui_styled::backdrop"
    }

    // Runs after all app UI, so placeholder shapes (panel fills etc.) are final.
    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let requests = take_pending(&ctx, ctx.viewport_id());
        if requests.is_empty() {
            return;
        }
        let ppp = ctx.pixels_per_point();
        let frame = ctx.cumulative_frame_nr();
        let renderer = self.renderer.read();
        for req in requests {
            let Some((rect, region, shapes)) = collect(&ctx, &req) else {
                continue;
            };
            let reuse = self
                .cache
                .get(&req.key)
                .filter(|c| c.matches(ppp, rect, region, &shapes))
                .map(|c| Arc::clone(&c.primitives));
            let mut changed = reuse.is_none();
            let mut primitives =
                reuse.unwrap_or_else(|| Arc::new(ctx.tessellate(shapes.clone(), ppp)));
            let mut textures = shared_textures(&renderer, &self.native, &primitives);
            self.cache.insert(
                req.key,
                Cached {
                    ppp,
                    rect,
                    region,
                    shapes,
                    primitives: Arc::clone(&primitives),
                    frame,
                },
            );
            // A scene changes every frame, so it always forces a redraw.
            if let Some(scene) = &self.scene {
                let mut with_scene = vec![scene_primitive(ctx.viewport_rect())];
                with_scene.extend(primitives.iter().cloned());
                primitives = Arc::new(with_scene);
                textures.push((SCENE_TEXTURE, scene.clone(), wgpu::FilterMode::Linear));
                changed = true;
            }
            *req.payload.lock().unwrap() = Payload {
                primitives,
                textures,
                rects: Some((rect, region)),
                changed,
            };
        }
        self.cache
            .retain(|_, c| frame.saturating_sub(c.frame) <= EVICT_AFTER_FRAMES);
    }
}

/// Container rect, backdrop region (both global points) and the shapes under the container.
pub(crate) fn collect(ctx: &Context, req: &Request) -> Option<(Rect, Rect, Vec<ClippedShape>)> {
    let layers = layers_up_to(
        &ctx.memory(|m| m.layer_ids().collect::<Vec<_>>()),
        req.layer,
    );
    // Read before `graphics`: the context lock is not re-entrant.
    let transforms: Vec<TSTransform> = layers
        .iter()
        .map(|&l| {
            ctx.layer_transform_to_global(l)
                .unwrap_or(TSTransform::IDENTITY)
        })
        .collect();

    ctx.graphics(|g| {
        let own = g.get(req.layer)?;
        let Shape::Callback(callback) = &own.all_entries().nth(req.slot.0)?.shape else {
            return None;
        };
        let rect = *transforms.last()? * callback.rect;
        let region = rect.expand(req.blur * CULL_FACTOR);

        let mut shapes = Vec::new();
        for (&layer, &t) in layers.iter().zip(&transforms) {
            let Some(list) = g.get(layer) else { continue };
            let take = if layer == req.layer {
                req.slot.0
            } else {
                usize::MAX
            };
            for s in list.all_entries().take(take) {
                if matches!(s.shape, Shape::Callback(_))
                    || !(t * s.clip_rect).intersects(region)
                    || !(t * s.shape.visual_bounding_rect()).intersects(region)
                {
                    continue;
                }
                let mut s = s.clone();
                if t != TSTransform::IDENTITY {
                    s.transform(t);
                }
                shapes.push(s);
            }
        }
        Some((rect, region, shapes))
    })
}

/// Layers drawn before and including `me`, in draw order. Mirrors `GraphicLayers::drain`:
/// by `Order`, then last frame's area order, then layers missing from it.
fn layers_up_to(area_order: &[LayerId], me: LayerId) -> Vec<LayerId> {
    let mut layers = area_order.to_vec();
    for extra in [LayerId::background(), me] {
        if !layers.contains(&extra) {
            layers.push(extra);
        }
    }
    let rank = |l: &LayerId| {
        let pos = area_order.iter().position(|a| a == l);
        (l.order as usize, pos.unwrap_or(usize::MAX))
    };
    layers.sort_by_key(rank);
    let me_idx = layers.iter().position(|&l| l == me).unwrap_or(0);
    layers.truncate(me_idx + 1);
    layers
}

/// Main-renderer textures used by `primitives`, so a private renderer can sample them.
/// Native textures have no `wgpu::Texture` to share and drop out here.
fn shared_textures(
    renderer: &Renderer,
    native: &HashMap<TextureId, wgpu::Texture>,
    primitives: &[ClippedPrimitive],
) -> Vec<(TextureId, wgpu::Texture, wgpu::FilterMode)> {
    let mut ids: Vec<TextureId> = primitives
        .iter()
        .filter_map(|p| match &p.primitive {
            Primitive::Mesh(m) => Some(m.texture_id),
            Primitive::Callback(_) => None,
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids.into_iter()
        .filter_map(|id| {
            let t = renderer.texture(&id)?;
            let filter = match t.options.map(|o| o.magnification) {
                Some(TextureFilter::Nearest) => wgpu::FilterMode::Nearest,
                _ => wgpu::FilterMode::Linear,
            };
            let texture = t.texture.clone().or_else(|| native.get(&id).cloned())?;
            Some((id, texture, filter))
        })
        .collect()
}

/// The scene as a textured quad over the whole viewport.
fn scene_primitive(viewport: Rect) -> ClippedPrimitive {
    let mut mesh = Mesh::with_texture(SCENE_TEXTURE);
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    mesh.add_rect_with_uv(viewport, uv, Color32::WHITE);
    ClippedPrimitive {
        clip_rect: viewport,
        primitive: Primitive::Mesh(mesh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Color32, Order, Pos2, Stroke, pos2, vec2};

    fn layer(order: Order, name: &str) -> LayerId {
        LayerId::new(order, Id::unique(name))
    }

    #[test]
    fn layers_up_to_sorts_by_order_then_area_order() {
        let fg = layer(Order::Foreground, "fg");
        let mid_a = layer(Order::Middle, "a");
        let mid_b = layer(Order::Middle, "b");
        let tooltip = layer(Order::Tooltip, "tip");
        let area_order = [fg, mid_b, tooltip, mid_a];

        assert_eq!(
            layers_up_to(&area_order, mid_a),
            vec![LayerId::background(), mid_b, mid_a]
        );
        assert_eq!(
            layers_up_to(&area_order, fg),
            vec![LayerId::background(), mid_b, mid_a, fg]
        );
    }

    #[test]
    fn layers_up_to_handles_a_layer_missing_from_last_frame() {
        let mid = layer(Order::Middle, "known");
        let new = layer(Order::Middle, "new");
        let fg = layer(Order::Foreground, "fg");
        assert_eq!(
            layers_up_to(&[mid, fg], new),
            vec![LayerId::background(), mid, new]
        );
    }

    fn raw() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        }
    }

    /// Paints a scene, queues a request for `glass` and returns what `collect` finds.
    fn run(
        ctx: &Context,
        blur: f32,
        scene: impl Fn(&mut Ui),
        glass_layer: LayerId,
        glass_rect: Rect,
    ) -> Option<(Rect, Rect, Vec<ClippedShape>)> {
        let mut out = None;
        // Second frame so area order from the first is known.
        for _ in 0..2 {
            let _ = crate::test_util::run_ui(ctx, raw(), |ui| {
                scene(ui);
                let painter = ui.ctx().layer_painter(glass_layer);
                let slot = painter.add(Shape::Noop);
                painter.circle_filled(glass_rect.center(), 4.0, Color32::RED);
                painter.set(
                    slot,
                    egui::PaintCallback {
                        rect: glass_rect,
                        callback: Arc::new(()),
                    },
                );
                let req = Request {
                    key: Id::NULL,
                    viewport: ui.ctx().viewport_id(),
                    layer: glass_layer,
                    slot,
                    blur,
                    payload: Arc::default(),
                };
                out = collect(ui.ctx(), &req);
            });
        }
        out
    }

    fn bounds(shapes: &[ClippedShape]) -> Vec<Rect> {
        shapes
            .iter()
            .map(|s| s.shape.visual_bounding_rect())
            .collect()
    }

    #[test]
    fn collects_lower_layers_and_own_layer_prefix_only() {
        let ctx = Context::default();
        let fg = layer(Order::Foreground, "glass");
        let above = layer(Order::Tooltip, "above");
        let below_rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(50.0, 50.0));
        let own_rect = Rect::from_min_size(pos2(120.0, 120.0), vec2(10.0, 10.0));
        let glass = Rect::from_min_size(pos2(90.0, 90.0), vec2(80.0, 80.0));

        let (rect, region, shapes) = run(
            &ctx,
            0.0,
            |ui| {
                ui.painter().rect_filled(below_rect, 0.0, Color32::BLUE);
                ui.ctx()
                    .layer_painter(fg)
                    .rect_filled(own_rect, 0.0, Color32::GREEN);
                ui.ctx()
                    .layer_painter(above)
                    .rect_filled(glass, 0.0, Color32::WHITE);
            },
            fg,
            glass,
        )
        .expect("callback found at slot");

        assert_eq!(rect, glass);
        assert_eq!(region, glass);
        // Background rect and the own-layer shape before the slot; not the circle
        // painted after it, the higher layer, or the callback itself.
        assert_eq!(bounds(&shapes), vec![below_rect, own_rect]);
    }

    #[test]
    fn culls_shapes_outside_the_blur_region() {
        let ctx = Context::default();
        let fg = layer(Order::Foreground, "glass");
        let near = Rect::from_min_size(pos2(200.0, 100.0), vec2(10.0, 10.0));
        let far = Rect::from_min_size(pos2(600.0, 100.0), vec2(10.0, 10.0));
        let glass = Rect::from_min_size(pos2(100.0, 100.0), vec2(50.0, 50.0));

        // Blur 20 reaches 80pt past the glass: `near` is inside, `far` is not.
        let (_, region, shapes) = run(
            &ctx,
            20.0,
            |ui| {
                ui.painter()
                    .line_segment([near.min, near.max], Stroke::new(1.0, Color32::BLUE));
                ui.painter()
                    .line_segment([far.min, far.max], Stroke::new(1.0, Color32::BLUE));
            },
            fg,
            glass,
        )
        .unwrap();

        assert_eq!(region, glass.expand(80.0));
        assert_eq!(shapes.len(), 1);
        assert!(bounds(&shapes)[0].intersects(near));
    }

    #[test]
    fn scene_quad_covers_the_viewport_with_full_uvs() {
        let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let p = scene_primitive(viewport);
        let Primitive::Mesh(mesh) = p.primitive else {
            panic!("scene should be a mesh");
        };
        assert_eq!(p.clip_rect, viewport);
        assert_eq!(mesh.texture_id, SCENE_TEXTURE);
        assert_eq!(mesh.calc_bounds(), viewport);
        let uvs: Vec<_> = mesh.vertices.iter().map(|v| v.uv).collect();
        assert!(uvs.contains(&pos2(0.0, 0.0)) && uvs.contains(&pos2(1.0, 1.0)));
    }

    #[test]
    fn user_textures_are_never_reused() {
        let font = Shape::rect_filled(
            Rect::from_min_size(Pos2::ZERO, vec2(4.0, 4.0)),
            0.0,
            Color32::RED,
        );
        let image = Shape::image(
            TextureId::User(3),
            Rect::from_min_size(Pos2::ZERO, vec2(4.0, 4.0)),
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        assert!(font_only(&font));
        assert!(!font_only(&image));
        assert!(!font_only(&Shape::Vec(vec![font, image])));
    }

    #[test]
    fn missing_callback_at_slot_collects_nothing() {
        let ctx = Context::default();
        let mut found = Some(());
        let _ = crate::test_util::run_ui(&ctx, raw(), |ui| {
            let slot = ui.painter().add(Shape::Noop);
            let req = Request {
                key: Id::NULL,
                viewport: ui.ctx().viewport_id(),
                layer: ui.layer_id(),
                slot,
                blur: 10.0,
                payload: Arc::default(),
            };
            found = collect(ui.ctx(), &req).map(|_| ());
        });
        assert!(found.is_none());
    }
}
