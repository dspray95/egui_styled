use egui::Id;

use crate::theme::StyledTheme;

fn theme_id() -> Id {
    Id::new("egui_styled::StyledTheme")
}

pub trait ThemeExt {
    fn set_styled_theme(&self, theme: StyledTheme);
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