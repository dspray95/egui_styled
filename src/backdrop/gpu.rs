//! Paint callback: render the collected backdrop offscreen, blur it, composite it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use egui::epaint::{ClippedPrimitive, Primitive};
use egui::{CornerRadius, Id, TextureId};
use egui_wgpu::wgpu::util::DeviceExt as _;
use egui_wgpu::{
    CallbackResources, CallbackTrait, Renderer, RendererOptions, ScreenDescriptor, wgpu,
};

use super::collect::{EVICT_AFTER_FRAMES, Payload};

const MAX_LEVELS: u32 = 7;

const BLUR_WGSL: &str = concat!(include_str!("prelude.wgsl"), include_str!("blur.wgsl"));
const COMPOSITE_WGSL: &str = concat!(include_str!("prelude.wgsl"), include_str!("composite.wgsl"));

/// Dual Kawase spreads by roughly `offset * 2^levels` px; pick both so that tracks `blur_px`.
pub(crate) fn blur_levels(blur_px: f32) -> (u32, f32) {
    let px = blur_px.max(2.0);
    let levels = (px.log2().floor() as u32).clamp(1, MAX_LEVELS);
    (levels, px / (1u32 << levels) as f32)
}

pub(crate) struct BackdropCallback {
    pub key: Id,
    pub frame: u64,
    pub blur: f32,
    pub corner_radius: CornerRadius,
    pub opacity: f32,
    pub payload: Arc<Mutex<Payload>>,
}

impl CallbackTrait for BackdropCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(res) = resources.get_mut::<GlassResources>() else {
            return Vec::new();
        };
        res.panels.retain(|k, p| {
            *k == self.key || self.frame.saturating_sub(p.frame) <= EVICT_AFTER_FRAMES
        });
        let shared = &res.shared;
        // One private renderer per container: queued buffer writes from a shared one would
        // all land before any pass runs, so every container would draw the last one's meshes.
        let panel = res
            .panels
            .entry(self.key)
            .or_insert_with(|| Panel::new(device, shared));
        panel.frame = self.frame;
        panel.ready = false;

        let Payload {
            primitives,
            textures,
            rects,
            changed,
        } = std::mem::take(&mut *self.payload.lock().unwrap());
        let Some((rect, region)) = rects else {
            return Vec::new();
        };

        // Clipped to the screen so edges clamp to screen content instead of fading to black.
        let ppp = screen.pixels_per_point;
        let [sw, sh] = screen.size_in_pixels.map(|v| v as f32);
        let x0 = (region.min.x * ppp).floor().clamp(0.0, sw);
        let y0 = (region.min.y * ppp).floor().clamp(0.0, sh);
        let x1 = (region.max.x * ppp).ceil().clamp(0.0, sw);
        let y1 = (region.max.y * ppp).ceil().clamp(0.0, sh);
        if x1 <= x0 || y1 <= y0 {
            return Vec::new();
        }
        let size = [(x1 - x0) as u32, (y1 - y0) as u32];
        let (levels, offset) = blur_levels(self.blur * ppp);
        panel.ensure_chain(device, shared, size, levels);

        // Same backdrop, region and blur as the last render: level 1 still holds the result.
        let render_key = (x0, y0, offset, ppp);
        let cmds = if !changed && panel.rendered == Some(render_key) {
            Vec::new()
        } else {
            panel.rendered = Some(render_key);
            let origin = egui::vec2(x0, y0) / ppp;
            panel.render(
                device,
                queue,
                encoder,
                shared,
                &primitives,
                textures,
                origin,
                offset,
                ppp,
            )
        };

        let r = self.corner_radius;
        queue.write_buffer(
            &panel.composite_uniform,
            0,
            &f32_bytes(&[
                rect.min.x * ppp,
                rect.min.y * ppp,
                rect.max.x * ppp,
                rect.max.y * ppp,
                r.nw as f32 * ppp,
                r.ne as f32 * ppp,
                r.se as f32 * ppp,
                r.sw as f32 * ppp,
                x0,
                y0,
                x1 - x0,
                y1 - y0,
                self.opacity,
                0.0,
                0.0,
                0.0,
            ]),
        );
        panel.ready = true;
        cmds
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let Some(res) = resources.get::<GlassResources>() else {
            return;
        };
        let Some(panel) = res.panels.get(&self.key) else {
            return;
        };
        if !panel.ready || panel.frame != self.frame {
            return;
        }
        let Some(bind_group) = &panel.composite else {
            return;
        };
        pass.set_pipeline(&res.shared.composite);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn begin_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    label: &str,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
}

fn f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_ne_bytes()).collect()
}

/// Lives in egui_wgpu's `CallbackResources`; inserted by [`super::install`].
pub(crate) struct GlassResources {
    shared: Shared,
    panels: HashMap<Id, Panel>,
}

impl GlassResources {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat, msaa: u32) -> Self {
        Self {
            shared: Shared::new(device, format, msaa),
            panels: HashMap::new(),
        }
    }
}

struct Shared {
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
}

impl Shared {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, msaa: u32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("backdrop_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("backdrop_pipeline_layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let blur = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("backdrop_blur"),
            source: wgpu::ShaderSource::Wgsl(BLUR_WGSL.into()),
        });
        let composite = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("backdrop_composite"),
            source: wgpu::ShaderSource::Wgsl(COMPOSITE_WGSL.into()),
        });
        let pipeline = |module: &wgpu::ShaderModule, fs: &str, blend, samples: u32| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(fs),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_full"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: samples,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            down: pipeline(&blur, "fs_down", None, 1),
            up: pipeline(&blur, "fs_up", None, 1),
            // Drawn inside egui's main pass, so it must match that pass's sample count.
            composite: pipeline(
                &composite,
                "fs_composite",
                Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                msaa,
            ),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("backdrop_sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            format,
            layout,
        }
    }

    fn bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        uniform: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("backdrop_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        })
    }
}

struct Level {
    size: [u32; 2],
    view: wgpu::TextureView,
}

struct BlurPass {
    src: usize,
    dst: usize,
    up: bool,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// GPU state for one blurred container.
struct Panel {
    renderer: Renderer,
    /// Main-renderer id to private id, plus the texture it was bound to.
    texture_ids: HashMap<TextureId, (TextureId, wgpu::Texture)>,
    chain_key: Option<([u32; 2], u32)>,
    /// What level 1 was last rendered for: region origin (px), Kawase offset, ppp.
    rendered: Option<(f32, f32, f32, f32)>,
    levels: Vec<Level>,
    passes: Vec<BlurPass>,
    composite_uniform: wgpu::Buffer,
    composite: Option<wgpu::BindGroup>,
    frame: u64,
    ready: bool,
}

impl Panel {
    fn new(device: &wgpu::Device, shared: &Shared) -> Self {
        Self {
            renderer: Renderer::new(
                device,
                shared.format,
                RendererOptions {
                    msaa_samples: 1,
                    dithering: false,
                    ..Default::default()
                },
            ),
            texture_ids: HashMap::new(),
            chain_key: None,
            rendered: None,
            levels: Vec::new(),
            passes: Vec::new(),
            composite_uniform: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("backdrop_composite_uniform"),
                contents: &[0; 64],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            composite: None,
            frame: 0,
            ready: false,
        }
    }

    /// Draws the backdrop into level 0 and blurs it down the chain into level 1.
    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        shared: &Shared,
        primitives: &[ClippedPrimitive],
        textures: Vec<(TextureId, wgpu::Texture, wgpu::FilterMode)>,
        origin: egui::Vec2,
        offset: f32,
        ppp: f32,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut remap = HashMap::new();
        for (id, texture, filter) in textures {
            let private = match self.texture_ids.get(&id) {
                // Rebinding allocates; skip it while the texture is unchanged.
                Some((p, bound)) if *bound == texture => *p,
                Some(&(p, _)) => {
                    let view = texture.create_view(&Default::default());
                    self.renderer
                        .update_egui_texture_from_wgpu_texture(device, &view, filter, p);
                    self.texture_ids.insert(id, (p, texture));
                    p
                }
                None => {
                    let view = texture.create_view(&Default::default());
                    let p = self.renderer.register_native_texture(device, &view, filter);
                    self.texture_ids.insert(id, (p, texture));
                    p
                }
            };
            remap.insert(id, private);
        }

        let primitives: Vec<ClippedPrimitive> = primitives
            .iter()
            .filter_map(|p| {
                let Primitive::Mesh(m) = &p.primitive else {
                    return None;
                };
                let &texture_id = remap.get(&m.texture_id)?;
                let mut m = m.clone();
                m.texture_id = texture_id;
                m.translate(-origin);
                Some(ClippedPrimitive {
                    clip_rect: p.clip_rect.translate(-origin),
                    primitive: Primitive::Mesh(m),
                })
            })
            .collect();

        let local = ScreenDescriptor {
            size_in_pixels: self.levels[0].size,
            pixels_per_point: ppp,
        };
        let cmds = self
            .renderer
            .update_buffers(device, queue, encoder, &primitives, &local);
        {
            let mut pass =
                begin_pass(encoder, &self.levels[0].view, "backdrop_scene").forget_lifetime();
            self.renderer.render(&mut pass, &primitives, &local);
        }
        for blur in &self.passes {
            let [w, h] = self.levels[blur.src].size.map(|v| v as f32);
            queue.write_buffer(
                &blur.uniform,
                0,
                &f32_bytes(&[0.5 / w, 0.5 / h, offset, 0.0]),
            );
            let mut pass = begin_pass(encoder, &self.levels[blur.dst].view, "backdrop_blur");
            pass.set_pipeline(if blur.up { &shared.up } else { &shared.down });
            pass.set_bind_group(0, &blur.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        cmds
    }

    /// Level 0 is the backdrop region; level `i` is `1/2^i` size. The result ends in level 1.
    fn ensure_chain(&mut self, device: &wgpu::Device, shared: &Shared, size: [u32; 2], n: u32) {
        if self.chain_key == Some((size, n)) {
            return;
        }
        self.chain_key = Some((size, n));
        self.rendered = None;
        self.levels = (0..=n)
            .map(|i| {
                let size = [(size[0] >> i).max(1), (size[1] >> i).max(1)];
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("backdrop_level"),
                    size: wgpu::Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: shared.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                Level {
                    size,
                    view: texture.create_view(&Default::default()),
                }
            })
            .collect();

        let n = n as usize;
        let downs = (1..=n).map(|i| (i - 1, i, false));
        let ups = (2..=n).rev().map(|i| (i, i - 1, true));
        self.passes = downs
            .chain(ups)
            .map(|(src, dst, up)| {
                let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("backdrop_blur_uniform"),
                    size: 16,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                BlurPass {
                    src,
                    dst,
                    up,
                    bind_group: shared.bind_group(device, &self.levels[src].view, &uniform),
                    uniform,
                }
            })
            .collect();
        self.composite =
            Some(shared.bind_group(device, &self.levels[1].view, &self.composite_uniform));
    }
}

#[cfg(test)]
mod tests {
    use super::blur_levels;

    #[test]
    fn blur_levels_track_radius_continuously() {
        let spread = |px: f32| {
            let (n, offset) = blur_levels(px);
            offset * (1u32 << n) as f32
        };
        for px in [2.0, 3.0, 7.9, 8.0, 8.1, 38.0, 100.0] {
            assert!((spread(px) - px).abs() < 1e-3, "px = {px}");
        }
        assert_eq!(blur_levels(0.0), (1, 1.0));
        assert_eq!(blur_levels(1000.0).0, 7);
    }
}
