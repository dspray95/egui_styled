use egui::{Id, Response, Ui};

#[derive(Clone, Copy, Default, Debug)]
pub struct PseudoState {
    pub hovered: bool,
    pub active: bool,  // Mouse down on widget
    pub focused: bool, // Keyboard focus (e.g. text editing)
}

/// We use egui::Id for the widgets unique key,
/// because it already exists
impl PseudoState {
    /// Read last frame's state from memory
    pub fn load(ui: &Ui, id: Id) -> Self {
        ui.memory(|mem| mem.data.get_temp::<PseudoState>(id).unwrap_or_default())
    }

    /// Write this frames state into memory for next frame
    pub fn store(self, ui: &Ui, id: Id) {
        ui.memory_mut(|mem| {
            mem.data.insert_temp(id, self);
        })
    }

    /// Build from an egui::Response after rendering
    pub fn from_response(response: &Response) -> Self {
        Self {
            hovered: response.hovered(),
            active: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
        }
    }
}
