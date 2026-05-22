use egui::{Context, Id};
use std::any::TypeId;

use crate::theme::StyledTheme;

fn slot_id() -> Id {
    Id::new("egui_styled::design_slot")
}

/// Typed storage for design data on [`egui::Context`].
///
/// One slot per `TypeId` — `set_design_data::<MyColors>(c)` overwrites any
/// previous `MyColors`, but doesn't touch other types like
/// [`StyledTheme`] or your own `AudioCues` / `SyntaxColors`. The crate uses
/// this primitive for [`StyledTheme`] itself; everything else is yours to
/// define and store.
///
/// If you need two slots of the same type (e.g., two `Vec<Color32>`
/// palettes), newtype them: `struct UiColors(Vec<Color32>)` vs
/// `struct DebugColors(Vec<Color32>)`.
pub trait DesignSlots {
    /// Replace the stored value of type `T`. Subsequent `design_data::<T>()`
    /// calls return this value until overwritten.
    fn set_design_data<T: 'static + Clone + Send + Sync>(&self, value: T);

    /// Read the stored value of type `T`. Returns `T::default()` if nothing
    /// has been stored under this type.
    fn design_data<T: 'static + Clone + Send + Sync + Default>(&self) -> T;

    /// Fetch the styled theme and an arbitrary additional design type in a
    /// single call. Removes the two-line ceremony at the top of every panel
    /// (`let theme = ...; let colors = ...;`).
    ///
    /// ```ignore
    /// let (theme, colors) = ui.ctx().design::<MyColors>();
    /// ```
    fn design<T: 'static + Clone + Send + Sync + Default>(&self) -> (StyledTheme, T) {
        (self.design_data::<StyledTheme>(), self.design_data::<T>())
    }
}

impl DesignSlots for Context {
    fn set_design_data<T: 'static + Clone + Send + Sync>(&self, value: T) {
        // egui's IdTypeMap keys by (Id, TypeId), so the same `slot_id`
        // can hold every distinct `T` without collision.
        let _ = TypeId::of::<T>(); // doc that we rely on TypeId
        self.memory_mut(|mem| mem.data.insert_temp(slot_id(), value));
    }

    fn design_data<T: 'static + Clone + Send + Sync + Default>(&self) -> T {
        self.memory_mut(|mem| mem.data.get_temp::<T>(slot_id()).unwrap_or_default())
    }
}

/// Convenience wrappers around [`DesignSlots`] for [`StyledTheme`] — so
/// beginners reading the README never have to encounter generics.
pub trait ThemeExt {
    /// Replace the current styled theme.
    fn set_styled_theme(&self, theme: StyledTheme);

    /// Read the current styled theme. Returns [`StyledTheme::default`] if
    /// none has been set.
    fn styled_theme(&self) -> StyledTheme;
}

impl ThemeExt for Context {
    fn set_styled_theme(&self, theme: StyledTheme) {
        self.set_design_data(theme);
    }

    fn styled_theme(&self) -> StyledTheme {
        self.design_data::<StyledTheme>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default, PartialEq, Debug)]
    struct FooColors {
        accent: u32,
    }

    #[derive(Clone, Default, PartialEq, Debug)]
    struct BarColors {
        accent: u32,
    }

    #[test]
    fn design_slot_roundtrip() {
        let ctx = Context::default();
        ctx.set_design_data(FooColors { accent: 42 });
        assert_eq!(ctx.design_data::<FooColors>().accent, 42);
    }

    #[test]
    fn different_types_dont_collide() {
        let ctx = Context::default();
        ctx.set_design_data(FooColors { accent: 1 });
        ctx.set_design_data(BarColors { accent: 2 });
        assert_eq!(ctx.design_data::<FooColors>().accent, 1);
        assert_eq!(ctx.design_data::<BarColors>().accent, 2);
    }

    #[test]
    fn unset_type_returns_default() {
        let ctx = Context::default();
        assert_eq!(ctx.design_data::<FooColors>(), FooColors::default());
    }

    #[test]
    fn theme_ext_still_works() {
        let ctx = Context::default();
        ctx.set_styled_theme(StyledTheme::default());
        let _ = ctx.styled_theme();
    }
}
