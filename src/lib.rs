pub mod color;
pub mod containers;
pub mod state;
pub mod style;
pub mod widgets;

pub use color::{rgb, rgba};

use crate::containers::column::StyledColumn;
use crate::containers::frame::StyledFrame;
use crate::containers::row::StyledRow;
use crate::widgets::button::StyledButton;
use crate::widgets::text_edit::StyledTextEdit;
use egui::WidgetText;

pub struct Styled;

impl Styled {
    pub fn frame() -> StyledFrame {
        StyledFrame::new()
    }

    pub fn button(text: impl Into<WidgetText>) -> StyledButton {
        StyledButton::new(text)
    }

    pub fn text_edit(text: &mut String) -> StyledTextEdit<'_> {
        StyledTextEdit::new(text)
    }

    pub fn row() -> StyledRow {
        StyledRow::new()
    }

    pub fn column() -> StyledColumn {
        StyledColumn::new()
    }
}
