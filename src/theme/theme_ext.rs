use egui::Id;

use crate::theme::StyledTheme;

fn theme_id() -> Id {
    Id::new("egui_styled::StyledTheme")
}

/// Context-attached accessors for the active [`StyledTheme`].
///
/// Implemented for [`egui::Context`]. Call `set_styled_theme` once at app
/// startup, then `ui.ctx().styled_theme()` anywhere you need the tokens.
///
/// Theme storage lives in [`egui::Memory`] under a crate-private id, so it
/// won't collide with other `Id::NULL` consumers.
pub trait ThemeExt {
    /// Replace the current styled theme. Subsequent `styled_theme()` calls
    /// return this value until overwritten.
    fn set_styled_theme(&self, theme: StyledTheme);

    /// Read the current styled theme. Returns [`StyledTheme::default`]
    /// (a deliberately bland fallback) if no theme has been set.
    fn styled_theme(&self) -> StyledTheme;
}

impl ThemeExt for egui::Context {
    fn set_styled_theme(&self, theme: StyledTheme) {
        self.memory_mut(|mem: &mut egui::Memory| {
            mem.data.insert_temp(theme_id(), theme);
        })
    }

    fn styled_theme(&self) -> StyledTheme {
        self.memory_mut(|mem| {
            mem.data
                .get_temp::<StyledTheme>(theme_id())
                .unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn set_and_get_roundtrip() {
        let ctx = egui::Context::default();
        let theme = StyledTheme {
            accent: Color32::from_rgb(123, 45, 67),
            ..Default::default()
        };
        ctx.set_styled_theme(theme);

        let read = ctx.styled_theme();
        assert_eq!(read.accent, Color32::from_rgb(123, 45, 67));
    }

    #[test]
    fn unset_theme_returns_default() {
        let ctx = egui::Context::default();
        let read = ctx.styled_theme();
        // Default theme is the neutral fallback — assert the accent matches it
        assert_eq!(read.accent, StyledTheme::default().accent);
    }

    #[test]
    fn set_overwrites_previous() {
        let ctx = egui::Context::default();
        ctx.set_styled_theme(StyledTheme {
            accent: Color32::RED,
            ..Default::default()
        });
        ctx.set_styled_theme(StyledTheme {
            accent: Color32::BLUE,
            ..Default::default()
        });

        assert_eq!(ctx.styled_theme().accent, Color32::BLUE);
    }
}
