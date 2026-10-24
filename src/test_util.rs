/// `Context::run_ui` that clears the textures delta; egui 0.37 panics when one is dropped unapplied.
pub(crate) fn run_ui(
    ctx: &egui::Context,
    raw: egui::RawInput,
    run_ui: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    let mut output = ctx.run_ui(raw, run_ui);
    output.textures_delta.clear();
    output
}
