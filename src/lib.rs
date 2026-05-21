pub mod apply;
pub mod color;
pub mod containers;
pub mod state;
pub mod style;
pub mod theme;
pub mod widgets;

pub use apply::Apply;
pub use color::{rgb, rgba};

use crate::containers::column::StyledColumn;
use crate::containers::frame::StyledFrame;
use crate::containers::row::StyledRow;
use crate::theme::StyledTheme;
use crate::widgets::button::StyledButton;
use crate::widgets::checkbox::StyledCheckbox;
use crate::widgets::combo_box::StyledComboBox;
use crate::widgets::label::StyledLabel;
use crate::widgets::slider::StyledSlider;
use crate::widgets::text_edit::StyledTextEdit;
use egui::{WidgetText, emath::Numeric};
use std::ops::RangeInclusive;

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

    pub fn theme() -> StyledTheme {
        StyledTheme::default()
    }

    pub fn label(text: impl Into<WidgetText>) -> StyledLabel {
        StyledLabel::new(text)
    }

    pub fn checkbox(checked: &mut bool, label: impl Into<WidgetText>) -> StyledCheckbox<'_> {
        StyledCheckbox::new(checked, label)
    }

    pub fn slider<T: Numeric>(value: &mut T, range: RangeInclusive<T>) -> StyledSlider<'_, T> {
        StyledSlider::new(value, range)
    }

    pub fn combo_box(
        id_source: impl std::hash::Hash,
        selected_text: impl Into<WidgetText>,
    ) -> StyledComboBox {
        StyledComboBox::new(id_source, selected_text)
    }
}
