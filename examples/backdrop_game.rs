//! Glass UI over a scene rendered with your own wgpu code (`--features backdrop-blur`).
//!
//! The scene is drawn into a texture each frame, shown with `ui.image` and handed to
//! `backdrop::register_native_texture` so glass can blur it. `EGUI_STYLED_SCENE=1` uses
//! `backdrop::set_scene_texture` instead, as an engine drawing outside egui would; eframe
//! then shows the scene only through the glass.
//! Set `EGUI_STYLED_SCREENSHOT=out.ppm` to save frame 60 and exit.

use eframe::egui_wgpu::{self, wgpu};
use egui::{Color32, Pos2, Rect, Ui, vec2};
use egui_styled::prelude::*;

const SCENE_WGSL: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex fn vs_full(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VsOut;
    o.pos = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    o.uv = uv;
    return o;
}

struct Scene { time: f32, aspect: f32, _pad: vec2<f32> };
@group(0) @binding(0) var<uniform> s: Scene;

// A neon grid with orbiting orbs: sharp detail for the blur to chew on.
@fragment fn fs_scene(in: VsOut) -> @location(0) vec4<f32> {
    let p = vec2<f32>(in.uv.x * s.aspect, in.uv.y);
    var col = mix(vec3<f32>(0.06, 0.02, 0.14), vec3<f32>(0.01, 0.08, 0.16), in.uv.y);
    // Distance to the nearest grid line in cells, faded over one pixel so scrolling lines
    // glide instead of snapping a whole pixel at a time.
    let cell = p * 10.0 + vec2<f32>(0.0, s.time * 0.2);
    let d = abs(fract(cell - 0.5) - 0.5) - 0.02;
    let px = fwidth(cell);
    let line = max(1.0 - smoothstep(0.0, px.x, d.x), 1.0 - smoothstep(0.0, px.y, d.y));
    col += vec3<f32>(0.0, 0.7, 0.9) * line * 0.6;
    for (var i = 0; i < 3; i++) {
        let fi = f32(i);
        let c = vec2<f32>(
            s.aspect * 0.5 + cos(s.time * 0.6 + fi * 2.1) * s.aspect * 0.3,
            0.5 + sin(s.time * 0.5 + fi * 1.7) * 0.3,
        );
        let d = length(p - c);
        col += vec3<f32>(1.0 - fi * 0.35, 0.25 + fi * 0.25, 0.7) * smoothstep(0.16, 0.14, d);
    }
    return vec4<f32>(col, 1.0);
}
"#;

/// Stands in for a game renderer: owns its texture and pipeline, draws once per frame.
struct Scene {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    /// Matches the window in pixels, so the scene is never stretched.
    size: [u32; 2],
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Scene {
    fn create_texture(device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // egui_wgpu expects native textures in this format.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn new(device: &wgpu::Device, size: [u32; 2]) -> Self {
        let texture = Self::create_texture(device, size);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene"),
            source: wgpu::ShaderSource::Wgsl(SCENE_WGSL.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_full"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_scene"),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene_uniform"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        Self {
            view: texture.create_view(&Default::default()),
            texture,
            size,
            pipeline,
            uniform,
            bind_group,
        }
    }

    /// Recreates the texture when `size` changes; returns whether it did.
    fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) -> bool {
        if size == self.size {
            return false;
        }
        self.texture = Self::create_texture(device, size);
        self.view = self.texture.create_view(&Default::default());
        self.size = size;
        true
    }

    fn draw(&self, device: &wgpu::Device, queue: &wgpu::Queue, time: f32) {
        let aspect = self.size[0] as f32 / self.size[1] as f32;
        let bytes: Vec<u8> = [time, aspect, 0.0, 0.0]
            .iter()
            .flat_map(|f| f.to_ne_bytes())
            .collect();
        queue.write_buffer(&self.uniform, 0, &bytes);
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
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
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
    }
}

struct App {
    render_state: egui_wgpu::RenderState,
    scene: Scene,
    /// `None` when the scene goes through `set_scene_texture` instead of `ui.image`.
    scene_id: Option<egui::TextureId>,
    volume: f32,
    frame: u64,
    screenshot: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("backdrop blur needs the wgpu backend");
        egui_styled::backdrop::install(&cc.egui_ctx, &render_state);
        let scene = Scene::new(&render_state.device, [1, 1]);

        let scene_id = if std::env::var("EGUI_STYLED_SCENE").is_ok() {
            egui_styled::backdrop::set_scene_texture(&cc.egui_ctx, Some(&scene.texture));
            None
        } else {
            let id = render_state.renderer.write().register_native_texture(
                &render_state.device,
                &scene.view,
                wgpu::FilterMode::Linear,
            );
            egui_styled::backdrop::register_native_texture(&cc.egui_ctx, id, &scene.texture);
            Some(id)
        };

        Self {
            render_state,
            scene,
            scene_id,
            volume: 0.7,
            frame: 0,
            screenshot: std::env::var("EGUI_STYLED_SCREENSHOT").ok(),
        }
    }

    /// A recreated texture has to be handed to egui_wgpu and the backdrop again.
    fn rebind_scene(&self, ctx: &egui::Context) {
        let rs = &self.render_state;
        match self.scene_id {
            Some(id) => {
                rs.renderer.write().update_egui_texture_from_wgpu_texture(
                    &rs.device,
                    &self.scene.view,
                    wgpu::FilterMode::Linear,
                    id,
                );
                egui_styled::backdrop::register_native_texture(ctx, id, &self.scene.texture);
            }
            None => egui_styled::backdrop::set_scene_texture(ctx, Some(&self.scene.texture)),
        }
    }

    fn hud(&mut self, ui: &mut Ui) {
        let glass = || {
            Styled::frame()
                .backdrop_blur(20.0)
                .bg(Color32::from_white_alpha(16))
                .border(1.0, Color32::from_white_alpha(50))
                .inner_glow(8.0, Color32::from_white_alpha(10))
                .corner_radius(18.0)
                .padding(20.0)
        };
        let screen = ui.max_rect();

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(Rect::from_min_size(
                screen.min + vec2(24.0, 24.0),
                vec2(280.0, 0.0),
            )),
            |ui| {
                glass().full_width().show(ui, |ui| {
                    Styled::column().gap(6.0).show(ui, |ui| {
                        Styled::label("PLAYER ONE")
                            .font_size(12.0)
                            .text_color(Color32::from_white_alpha(150))
                            .show(ui);
                        Styled::label("Score 128,450")
                            .font_size(22.0)
                            .bold()
                            .text_color(Color32::WHITE)
                            .show(ui);
                    });
                });
            },
        );

        let menu = Rect::from_center_size(screen.center(), vec2(340.0, 0.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(menu), |ui| {
            glass().full_width().show(ui, |ui| {
                Styled::column().gap(12.0).show(ui, |ui| {
                    Styled::label("Paused")
                        .font_size(28.0)
                        .bold()
                        .text_color(Color32::WHITE)
                        .show(ui);
                    Styled::slider(&mut self.volume, 0.0..=1.0)
                        .text("Volume")
                        .show(ui);
                    for label in ["Resume", "Settings", "Quit"] {
                        Styled::button(label)
                            .full_width()
                            .backdrop_blur(10.0)
                            .bg(Color32::from_white_alpha(20))
                            .hover_bg(Color32::from_white_alpha(40))
                            .border(1.0, Color32::from_white_alpha(40))
                            .text_color(Color32::WHITE)
                            .corner_radius(10.0)
                            .padding(10.0)
                            .show(ui);
                    }
                });
            });
        });
    }

    fn handle_screenshot(&mut self, ui: &Ui) {
        let Some(path) = self.screenshot.clone() else {
            return;
        };
        if self.frame != 60 {
            return;
        }
        let callback = egui::ScreenshotCallback::new(move |image| {
            let [w, h] = image.size;
            let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
            for p in &image.pixels {
                out.extend_from_slice(&[p.r(), p.g(), p.b()]);
            }
            std::fs::write(&path, out).expect("write screenshot");
            std::process::exit(0);
        });
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint();
        self.frame += 1;
        self.handle_screenshot(ui);

        let size = (ui.ctx().content_rect().size() * ui.ctx().pixels_per_point()).round();
        if self.scene.resize(
            &self.render_state.device,
            [size.x.max(1.0) as u32, size.y.max(1.0) as u32],
        ) {
            self.rebind_scene(ui.ctx());
        }

        // Submitted before egui's frame, so the texture is ready when egui samples it.
        let time = ui.input(|i| i.time) as f32;
        self.scene
            .draw(&self.render_state.device, &self.render_state.queue, time);

        // With a scene texture, anything opaque egui paints would cover it in the backdrop.
        let fill = if self.scene_id.is_some() {
            Color32::BLACK
        } else {
            Color32::TRANSPARENT
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(fill))
            .show(ui, |ui| {
                if let Some(id) = self.scene_id {
                    ui.painter().image(
                        id,
                        ui.max_rect(),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                self.hud(ui);
            });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "egui_styled glass over a game scene",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 720.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
