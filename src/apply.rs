/// Composable style functions for styled builders.
///
/// `apply` is just `f(self)` - the value of the trait is making method
/// chains read left-to-right when mixing builder calls with reusable style
/// functions:
///
/// ```ignore
/// Styled::button("Save")
///     .apply(primary_button(&theme))   // reusable preset
///     .margin_top(8.0)                 // per-call tweak
///     .show(ui);
/// ```
///
/// Implemented for every styled type via `impl_style_builders!`. A style
/// function typically returns a closure pre-bound to the theme - see the
/// README's "Composing styles" section for the full pattern.
pub trait Apply: Sized {
    /// Apply a style function to this builder.
    fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}
