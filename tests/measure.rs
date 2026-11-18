//! Measured containers must render the same whether or not every frame reruns.

use egui::{Align, Align2, Color32, Ui, pos2};
use egui_kittest::Harness;
use egui_styled::Styled;

fn scene(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        Styled::stack()
            .layer(|ui| {
                Styled::frame()
                    .bg(Color32::from_gray(40))
                    .padding(12.0)
                    .show(ui, |ui| {
                        ui.label("stack background with some width");
                    });
            })
            .layer_aligned(Align2::CENTER_CENTER, |ui| {
                Styled::row()
                    .gap(4.0)
                    .wrap()
                    .item(|ui| {
                        Styled::button("one").show(ui);
                    })
                    .item(|ui| {
                        Styled::button("two").show(ui);
                    })
                    .show(ui);
            })
            .show(ui);
        Styled::row()
            .full_width()
            .space_between()
            .item(|ui| {
                Styled::button("left").show(ui);
            })
            .item(|ui| {
                Styled::button("right").show(ui);
            })
            .show(ui);
        Styled::frame()
            .padding(6.0)
            .bg(Color32::DARK_BLUE)
            .show(ui, |ui| {
                ui.label("shrink to fit");
            });
        Styled::column()
            .full_width()
            .min_height(80.0)
            .full_height()
            .justify(Align::Center)
            .bg(Color32::from_gray(25))
            .show(ui, |ui| {
                ui.label("justified");
            });
    });
    Styled::area()
        .id("centred")
        .fixed_pos_centered(pos2(240.0, 300.0))
        .bg(Color32::DARK_RED)
        .padding(4.0)
        .show(ui.ctx(), |ui| {
            ui.label("centred area");
        });
}

fn render(force_multipass: bool) -> image::RgbaImage {
    let mut harness = Harness::builder().with_size((480.0, 360.0)).build_ui(scene);
    Styled::debug_force_multipass(&harness.ctx, force_multipass);
    harness.run();
    harness.render().expect("render")
}

#[test]
fn force_multipass_renders_identically() {
    let normal = render(false);
    let forced = render(true);
    assert!(
        normal.as_raw() == forced.as_raw(),
        "rendering differs under multipass"
    );
}
