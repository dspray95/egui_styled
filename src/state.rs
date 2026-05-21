use egui::{Id, Response, Ui};

/// Per-widget interaction state, carried frame-to-frame through
/// [`egui::Memory`].
///
/// This is how `egui_styled` implements `hover_bg` / `focus_border` /
/// `active_bg` without modifying egui itself: each frame, a widget loads
/// last frame's state, resolves its style against it, renders, then stores
/// the new state for next frame. The one-frame lag on state transitions is
/// imperceptible in practice.
#[derive(Clone, Copy, Default, Debug)]
pub struct PseudoState {
    /// Pointer is over the widget.
    pub hovered: bool,
    /// Pointer button is pressed on the widget (e.g. mid-click).
    pub active: bool,
    /// Widget owns keyboard focus (e.g. text editing).
    pub focused: bool,
}

impl PseudoState {
    /// Read last frame's pseudo-state for this widget id from
    /// [`egui::Memory`]. Returns `Self::default()` if nothing was stored.
    pub fn load(ui: &Ui, id: Id) -> Self {
        ui.memory(|mem| mem.data.get_temp::<PseudoState>(id).unwrap_or_default())
    }

    /// Write this frame's pseudo-state to [`egui::Memory`] for next frame
    /// to read.
    pub fn store(self, ui: &Ui, id: Id) {
        ui.memory_mut(|mem| {
            mem.data.insert_temp(id, self);
        })
    }

    /// Build a `PseudoState` from a widget's render-time [`Response`].
    pub fn from_response(response: &Response) -> Self {
        Self {
            hovered: response.hovered(),
            active: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_roundtrip() {
        let ctx = egui::Context::default();
        let id = Id::new("ps-test");
        let state = PseudoState {
            hovered: true,
            active: false,
            focused: true,
        };
        ctx.memory_mut(|mem| mem.data.insert_temp(id, state));
        let loaded = ctx.memory_mut(|mem| mem.data.get_temp::<PseudoState>(id).unwrap_or_default());
        assert!(loaded.hovered);
        assert!(!loaded.active);
        assert!(loaded.focused);
    }

    #[test]
    fn unset_id_returns_default() {
        let ctx = egui::Context::default();
        let id = Id::new("ps-never-stored");
        let loaded = ctx.memory_mut(|mem| mem.data.get_temp::<PseudoState>(id).unwrap_or_default());
        assert!(!loaded.hovered);
        assert!(!loaded.active);
        assert!(!loaded.focused);
    }
}
