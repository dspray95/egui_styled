use egui::Ui;

/// Reruns every frame, so code that isn't pass-safe breaks reliably.
pub(super) struct ForceMultipass;

impl egui::Plugin for ForceMultipass {
    fn debug_name(&self) -> &'static str {
        "egui_styled::force_multipass"
    }

    fn on_begin_pass(&mut self, ui: &mut Ui) {
        let context = ui.ctx();
        if context.current_pass_index() == 0 && super::with_state(context, |s| s.force_multipass) {
            context.request_discard("egui_styled debug_force_multipass");
        }
    }
}
