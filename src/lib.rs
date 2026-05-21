//! Tailwind-style utility styling for an immediate-mode toolkit.
//!
//! `egui_styled` wraps egui's widgets with builder APIs that handle the
//! per-widget `visuals_mut` dance for you, plus a small design-token theme
//! system. See the README for a Before/After comparison.
//!
//! ## Quick start
//!
//! ```ignore
//! use egui_styled::prelude::*;
//!
//! Styled::button("Save")
//!     .bg(theme.accent)
//!     .hover_bg(theme.accent_hover)
//!     .text_color(theme.fg_on_accent)
//!     .corner_radius(theme.rounding_md)
//!     .show(ui);
//! ```
//!
//! The [`prelude`] module re-exports everything you usually need.

pub mod apply;
pub mod color;
pub mod containers;
pub mod prelude;
pub mod state;
pub mod style;
pub mod theme;
pub mod widgets;

pub use apply::Apply;
pub use color::{rgb, rgba};
pub use containers::column::StyledColumn;
pub use containers::frame::StyledFrame;
pub use containers::row::StyledRow;
pub use theme::StyledTheme;
pub use theme::theme_ext::{DesignSlots, ThemeExt};
pub use theme::web_palette::WebPalette;
pub use widgets::button::StyledButton;
pub use widgets::checkbox::StyledCheckbox;
pub use widgets::combo_box::StyledComboBox;
pub use widgets::label::StyledLabel;
pub use widgets::slider::StyledSlider;
pub use widgets::text_edit::StyledTextEdit;

use egui::{WidgetText, emath::Numeric};
use std::ops::RangeInclusive;

/// Entry-point namespace for constructing styled widgets and containers.
///
/// `Styled::button("Save")` returns a [`StyledButton`] builder; the same
/// shorthand exists for every styled type. This keeps call sites short and
/// consistent (no `StyledButton::new`, no `StyledFrame::new`, etc).
pub struct Styled;

impl Styled {
    /// Start a styled frame (background, padding, border, corner radius).
    pub fn frame() -> StyledFrame {
        StyledFrame::new()
    }

    /// Start a styled button with per-state hover / active colors.
    pub fn button(text: impl Into<WidgetText>) -> StyledButton {
        StyledButton::new(text)
    }

    /// Start a styled single-line text edit. Use `.multiline()` for multi-line.
    pub fn text_edit(text: &mut String) -> StyledTextEdit<'_> {
        StyledTextEdit::new(text)
    }

    /// Start a styled horizontal layout. Pair with `.gap(...)` for spacing.
    pub fn row() -> StyledRow {
        StyledRow::new()
    }

    /// Start a styled vertical layout. Pair with `.gap(...)` for spacing.
    pub fn column() -> StyledColumn {
        StyledColumn::new()
    }

    /// Build a fresh default theme. Prefer `ctx.styled_theme()` to read the
    /// active app theme.
    pub fn theme() -> StyledTheme {
        StyledTheme::default()
    }

    /// Start a styled label. Themed colors / sizes via builder methods.
    pub fn label(text: impl Into<WidgetText>) -> StyledLabel {
        StyledLabel::new(text)
    }

    /// Start a styled checkbox. `checked` is mutated on click.
    pub fn checkbox(checked: &mut bool, label: impl Into<WidgetText>) -> StyledCheckbox<'_> {
        StyledCheckbox::new(checked, label)
    }

    /// Start a styled numeric slider. Generic over `T: Numeric`.
    pub fn slider<T: Numeric>(value: &mut T, range: RangeInclusive<T>) -> StyledSlider<'_, T> {
        StyledSlider::new(value, range)
    }

    /// Start a styled combo box. `id_source` must be unique within the parent ui.
    pub fn combo_box(
        id_source: impl std::hash::Hash,
        selected_text: impl Into<WidgetText>,
    ) -> StyledComboBox {
        StyledComboBox::new(id_source, selected_text)
    }
}
