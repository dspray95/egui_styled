use egui::{InnerResponse, Response, Ui};

/// Self-contained styled elements that render in one call: no body closure,
/// a plain [`Response`].
///
/// Every leaf widget (`StyledButton`, `StyledLabel`, ...) and every
/// pre-populated container (`StyledSpacer`, `DistributedRow`, `WrappingRow`,
/// `StyledStack`) implements this the same way, so generic helpers can accept
/// "any styled thing that just needs a `Ui`":
///
/// ```ignore
/// fn render<W: StyledWidget>(w: W, ui: &mut egui::Ui) -> egui::Response {
///     w.show(ui)
/// }
/// ```
///
/// [`StyledArea`](crate::StyledArea) and [`StyledComboBox`](crate::StyledComboBox)
/// don't implement this: `StyledArea` renders against `&Context` (it's a
/// floating layer outside the current `Ui` tree) and `StyledComboBox` returns
/// `InnerResponse<Option<()>>` from its menu-contents closure, mirroring
/// egui's own `ComboBox::show_ui`.
pub trait StyledWidget {
    fn show(self, ui: &mut Ui) -> Response;
}

/// Styled containers that wrap caller-provided content: a body closure in,
/// an [`InnerResponse<R>`] out.
///
/// `StyledFrame`, `StyledRow`, and `StyledColumn` all implement this the same
/// way, so generic helpers can accept "any styled box that lays out a body":
///
/// ```ignore
/// fn render<C: StyledContainer, R>(
///     c: C,
///     ui: &mut egui::Ui,
///     body: impl FnOnce(&mut egui::Ui) -> R,
/// ) -> egui::InnerResponse<R> {
///     c.show(ui, body)
/// }
/// ```
///
/// [`StyledArea`](crate::StyledArea) doesn't implement this despite having the
/// same body-closure shape: it takes `&Context` instead of `&mut Ui`, since the
/// underlying `egui::Area` is a floating layer outside the current `Ui` tree.
pub trait StyledContainer {
    fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;
}

/// Implements [`StyledWidget`] for a type that already has an inherent
/// `show(self, ui: &mut Ui) -> Response` method.
#[macro_export]
macro_rules! impl_styled_widget {
    ([$($gen:tt)*], $ty:ty) => {
        impl <$($gen)*> $crate::StyledWidget for $ty {
            fn show(self, ui: &mut egui::Ui) -> egui::Response {
                <$ty>::show(self, ui)
            }
        }
    };
    ($ty:ty) => {
        impl $crate::StyledWidget for $ty {
            fn show(self, ui: &mut egui::Ui) -> egui::Response {
                <$ty>::show(self, ui)
            }
        }
    };
}

/// Implements [`StyledContainer`] for a type that already has an inherent
/// `show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>` method.
#[macro_export]
macro_rules! impl_styled_container {
    ([$($gen:tt)*], $ty:ty) => {
        impl <$($gen)*> $crate::StyledContainer for $ty {
            fn show<R>(self, ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
                <$ty>::show(self, ui, body)
            }
        }
    };
    ($ty:ty) => {
        impl $crate::StyledContainer for $ty {
            fn show<R>(self, ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
                <$ty>::show(self, ui, body)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    //! Compile-time checklist: every public builder type must land in exactly
    //! one of `StyledWidget` / `StyledContainer`, or be named here as an
    //! intentional exception with a reason. Adding a new styled type without
    //! updating this list (or the type's `show` signature drifting from the
    //! shape a trait expects) fails `cargo test --lib` at compile time rather
    //! than silently reintroducing the five-shapes-for-`.show()` problem this
    //! module was written to close.
    use crate::{
        DistributedRow, StyledButton, StyledCheckbox, StyledColumn, StyledContainer, StyledFrame,
        StyledImage, StyledLabel, StyledRow, StyledSlider, StyledSpacer, StyledStack,
        StyledTextEdit, StyledWidget, WrappingRow,
    };

    fn assert_widget<T: StyledWidget>() {}
    fn assert_container<T: StyledContainer>() {}

    #[test]
    fn every_widget_and_container_implements_its_show_trait() {
        // `StyledWidget`: no body closure, plain `Response`.
        assert_widget::<StyledButton>();
        assert_widget::<StyledCheckbox<'_>>();
        assert_widget::<StyledSlider<'_, f32>>();
        assert_widget::<StyledLabel>();
        assert_widget::<StyledTextEdit<'_>>();
        assert_widget::<StyledImage>();
        assert_widget::<StyledSpacer>();
        assert_widget::<DistributedRow<'_>>();
        assert_widget::<WrappingRow<'_>>();
        assert_widget::<StyledStack<'_>>();

        // `StyledContainer`: body closure in, `InnerResponse<R>` out.
        assert_container::<StyledFrame>();
        assert_container::<StyledRow>();
        assert_container::<StyledColumn>();

        // Intentionally not in either trait (documented on their own `show`):
        // - `StyledArea` takes `&Context`, not `&mut Ui`.
        // - `StyledComboBox` returns `InnerResponse<Option<()>>` because the
        //   body closure only runs when the dropdown is open.
    }
}
