pub mod color;
pub mod containers;
pub mod state;
pub mod style;
pub mod widgets;

pub use color::{rgb, rgba};

use crate::containers::frame::StyledFrame;
use crate::widgets::button::StyledButton;
use egui::WidgetText;

pub struct Styled;

impl Styled {
    pub fn frame() -> StyledFrame {
        StyledFrame::new()
    }

    pub fn button(text: impl Into<WidgetText>) -> StyledButton {
        StyledButton::new(text)
    }
}
