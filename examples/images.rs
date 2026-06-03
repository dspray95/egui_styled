use egui::{CentralPanel, Color32, Vec2, vec2};
use egui_styled::prelude::*;

// Build a small 16×16 chequered texture at startup. This keeps the example
// self-contained (no external image files needed). In a real app you would use
// `egui::include_image!("path/to/image.png")` or
// `egui_extras::install_image_loaders(ctx)` for URI / file-based images.
fn make_checkerboard(ctx: &egui::Context) -> egui::TextureHandle {
    let size = [32, 32];
    let mut pixels = vec![Color32::TRANSPARENT; 32 * 32];
    for y in 0..32usize {
        for x in 0..32usize {
            pixels[y * 32 + x] = if (x / 8 + y / 8) % 2 == 0 {
                Color32::from_rgb(255, 140, 0)
            } else {
                Color32::WHITE
            };
        }
    }
    ctx.load_texture(
        "checkerboard",
        egui::ColorImage::new(size, pixels),
        egui::TextureOptions::NEAREST,
    )
}

fn images_ui(ctx: &egui::Context, handle: &egui::TextureHandle) {
    let sized = egui::load::SizedTexture::from_handle(handle);
    let src = || egui::ImageSource::Texture(sized);

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("egui_styled - Images");
        ui.add_space(16.0);

        // -----------------------------------------------------------------
        // 1. StyledImage - inline widget in the layout flow
        // -----------------------------------------------------------------
        ui.label("StyledImage - exact size, themed border + corner radius:");
        ui.add_space(8.0);
        Styled::image(src())
            .size(Vec2::splat(64.0))
            .corner_radius(12.0)
            .border(2.0, Color32::WHITE)
            .show(ui);

        ui.add_space(16.0);

        // Shadow
        ui.label("StyledImage - drop shadow:");
        ui.add_space(8.0);
        Styled::image(src())
            .size(Vec2::splat(64.0))
            .corner_radius(8.0)
            .shadow_filled(vec2(4.0, 4.0), Color32::from_black_alpha(120))
            .show(ui);

        ui.add_space(16.0);

        // Hover tint
        ui.label("StyledImage - hover tint (hover me):");
        ui.add_space(8.0);
        Styled::image(src())
            .size(Vec2::splat(64.0))
            .corner_radius(8.0)
            .border(1.0, Color32::from_gray(100))
            .hover_tint(Color32::from_rgba_unmultiplied(100, 160, 255, 200))
            .id("hover_tint_demo")
            .show(ui);

        ui.add_space(16.0);

        // Circle clip via full corner radius
        ui.label("StyledImage - circle clip (corner_radius = half size):");
        ui.add_space(8.0);
        Styled::image(src())
            .size(Vec2::splat(64.0))
            .corner_radius(32.0)
            .border(2.0, Color32::from_gray(160))
            .show(ui);

        ui.add_space(32.0);

        // -----------------------------------------------------------------
        // 2. background_image - texture behind children in a container
        // -----------------------------------------------------------------
        ui.label("background_image on Styled::frame - stretch fill, rounded corners:");
        ui.add_space(8.0);
        Styled::frame()
            .background_image(src())
            .corner_radius(16.0)
            .padding(24.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Text over a textured background")
                        .color(Color32::WHITE)
                        .strong(),
                );
                ui.add_space(8.0);
                Styled::button("Button inside textured frame")
                    .bg(Color32::from_black_alpha(160))
                    .hover_bg(Color32::from_black_alpha(210))
                    .text_color(Color32::WHITE)
                    .corner_radius(6.0)
                    .show(ui);
            });

        ui.add_space(16.0);

        // bg fill shows while texture loads; texture paints on top when ready.
        ui.label("background_image + tint + bg fill (solid fallback while loading):");
        ui.add_space(8.0);
        Styled::frame()
            .bg(Color32::from_rgb(40, 40, 60))
            .background_image(src())
            .background_image_tint(Color32::from_rgba_unmultiplied(255, 255, 255, 180))
            .corner_radius(12.0)
            .border(1.0, Color32::from_gray(80))
            .padding(20.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Semi-transparent texture over a solid fill")
                        .color(Color32::WHITE),
                );
            });

        ui.add_space(16.0);

        // Cover fit - aspect-ratio-preserving crop.
        ui.label("background_image - Cover fit (crop, not stretch) on a wide frame:");
        ui.add_space(8.0);
        Styled::frame()
            .full_width()
            .background_image(src())
            .background_image_fit(BackgroundImageFit::Cover)
            .corner_radius(8.0)
            .min_height(80.0)
            .padding(16.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Wide frame, Cover fit preserves aspect ratio")
                        .color(Color32::WHITE),
                );
            });
    });
}

fn main() -> eframe::Result<()> {
    let mut handle: Option<egui::TextureHandle> = None;

    eframe::run_ui_native(
        "egui_styled images",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 700.0]),
            ..Default::default()
        },
        move |ctx, _| {
            let h = handle.get_or_insert_with(|| make_checkerboard(ctx));
            images_ui(ctx, h);
        },
    )
}
