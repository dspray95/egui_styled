//! Backdrop blur for widgets and containers: CSS `backdrop-filter: blur()` on wgpu.
//!
//! Call [`install`] once at startup, then set `.backdrop_blur(radius)` on any styled widget
//! or container and give it a translucent `bg` as the tint:
//!
//! ```ignore
//! // In eframe's app creator:
//! egui_styled::backdrop::install(&cc.egui_ctx, cc.wgpu_render_state.as_ref().unwrap());
//!
//! Styled::frame()
//!     .backdrop_blur(24.0)
//!     .bg(Color32::from_white_alpha(18))
//!     .corner_radius(16.0)
//!     .show(ui, |ui| { /* … */ });
//! ```
//!
//! At the end of each pass, the shapes painted under each blurred element are re-tessellated,
//! drawn offscreen, blurred (dual Kawase) and composited under its fill. Nothing reads the
//! framebuffer, so an element never blurs itself.
//!
//! Without [`install`] the builder is a no-op and only the tint shows.
//!
//! # Limitations
//!
//! - Paint callbacks are not blurred. Images registered with
//!   `egui_wgpu::Renderer::register_native_texture` need [`register_native_texture`] too, and
//!   scenes drawn outside egui need [`set_scene_texture`].
//! - A blurred element does not see another blurred element's blur below it.
//! - Layers are ordered by last frame's area order, as egui itself does.
//! - wgpu only: native, WebGPU and WebGL2. Not the glow backend.

mod collect;
mod gpu;

use std::sync::Arc;

use egui::layers::ShapeIdx;
use egui::{CornerRadius, Rect, Ui};

/// Enables backdrop blur. `render_state` is eframe's `CreationContext::wgpu_render_state`.
pub fn install(ctx: &egui::Context, render_state: &egui_wgpu::RenderState) {
    install_with_msaa(ctx, render_state, 1);
}

/// [`install`] for apps with MSAA on. `samples` must match `NativeOptions::multisampling`.
pub fn install_with_msaa(ctx: &egui::Context, render_state: &egui_wgpu::RenderState, samples: u32) {
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(gpu::GlassResources::new(
            &render_state.device,
            render_state.target_format,
            samples.max(1),
        ));
    ctx.add_plugin(collect::Collector::new(Arc::clone(&render_state.renderer)));
}

/// Lets backdrops blur an image you registered with
/// `egui_wgpu::Renderer::register_native_texture`, which keeps no handle egui_styled can sample.
/// Pass the same `id` it returned. Call again if you recreate the texture.
pub fn register_native_texture(
    ctx: &egui::Context,
    id: egui::TextureId,
    texture: &egui_wgpu::wgpu::Texture,
) {
    ctx.with_plugin(|c: &mut collect::Collector| c.native.insert(id, texture.clone()));
}

/// Undoes [`register_native_texture`], e.g. after freeing the texture.
pub fn unregister_native_texture(ctx: &egui::Context, id: egui::TextureId) {
    ctx.with_plugin(|c: &mut collect::Collector| c.native.remove(&id));
}

/// Blurs `texture` under everything egui draws, stretched over the viewport. For scenes
/// rendered outside egui, e.g. straight to the surface before egui. `None` clears it.
pub fn set_scene_texture(ctx: &egui::Context, texture: Option<&egui_wgpu::wgpu::Texture>) {
    ctx.with_plugin(|c: &mut collect::Collector| c.scene = texture.cloned());
}

/// Turns the `slot` reserved under a container into its blurred backdrop.
pub(crate) fn queue(ui: &Ui, slot: ShapeIdx, rect: Rect, corner_radius: CornerRadius, blur: f32) {
    let ctx = ui.ctx();
    if ctx.plugin_opt::<collect::Collector>().is_none() {
        return;
    }
    let payload = Arc::default();
    let key = ui
        .make_persistent_id(ui.next_auto_id())
        .with(("__backdrop", ctx.viewport_id()));
    collect::push(
        ctx,
        collect::Request {
            key,
            viewport: ctx.viewport_id(),
            layer: ui.layer_id(),
            slot,
            blur,
            payload: Arc::clone(&payload),
        },
    );
    ui.painter().set(
        slot,
        egui_wgpu::Callback::new_paint_callback(
            rect,
            gpu::BackdropCallback {
                key,
                frame: ctx.cumulative_frame_nr(),
                blur,
                corner_radius,
                // egui fades shapes by alpha but can't fade a callback, so the shader does.
                opacity: ui.opacity(),
                payload,
            },
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use egui::Shape;

    #[test]
    fn without_install_backdrop_blur_paints_no_callback() {
        let ctx = egui::Context::default();
        let mut callbacks = usize::MAX;
        let _ = crate::test_util::run_ui(&ctx, Default::default(), |ui| {
            Styled::frame()
                .backdrop_blur(20.0)
                .bg(egui::Color32::from_white_alpha(20))
                .show(ui, |ui| ui.label("glass"));
            callbacks = ui.ctx().graphics(|g| {
                g.get(ui.layer_id()).map_or(0, |l| {
                    l.all_entries()
                        .filter(|s| matches!(s.shape, Shape::Callback(_)))
                        .count()
                })
            });
        });
        assert_eq!(callbacks, 0);
    }
}
