//! Bundled OFL fonts. Every weight is registered as its own family, since egui has no
//! synthetic bold: `font(Weight::Bold, 24.0)` picks the real 700 cut.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Inter,
    Geist,
    SpaceGrotesk,
}

impl Choice {
    pub const ALL: [Choice; 3] = [Choice::Inter, Choice::Geist, Choice::SpaceGrotesk];

    pub fn name(self) -> &'static str {
        match self {
            Choice::Inter => "Inter",
            Choice::Geist => "Geist",
            Choice::SpaceGrotesk => "Space Grotesk",
        }
    }

    /// Font-data keys for 400, 500, 600 and 700, then mono 400 and 600.
    fn keys(self) -> ([&'static str; 4], [&'static str; 2]) {
        match self {
            Choice::Inter => (
                ["inter-400", "inter-500", "inter-600", "inter-700"],
                ["jetbrains-mono-400", "jetbrains-mono-600"],
            ),
            Choice::Geist => (
                ["geist-400", "geist-500", "geist-600", "geist-700"],
                ["geist-mono-400", "geist-mono-600"],
            ),
            Choice::SpaceGrotesk => (
                [
                    "space-grotesk-400",
                    "space-grotesk-500",
                    "space-grotesk-600",
                    "space-grotesk-700",
                ],
                ["jetbrains-mono-400", "jetbrains-mono-600"],
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Weight {
    Regular,
    Medium,
    Semibold,
    Bold,
    Mono,
    MonoBold,
}

impl Weight {
    fn family(self) -> &'static str {
        match self {
            Weight::Regular => "w400",
            Weight::Medium => "w500",
            Weight::Semibold => "w600",
            Weight::Bold => "w700",
            Weight::Mono => "mono400",
            Weight::MonoBold => "mono600",
        }
    }
}

pub fn font(weight: Weight, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(weight.family().into()))
}

macro_rules! bundled {
    ($($key:literal => $path:literal),* $(,)?) => {
        [$(($key, include_bytes!($path) as &'static [u8])),*]
    };
}

const FONTS: [(&str, &[u8]); 16] = bundled! {
    "inter-400" => "assets/fonts/inter/inter-400.ttf",
    "inter-500" => "assets/fonts/inter/inter-500.ttf",
    "inter-600" => "assets/fonts/inter/inter-600.ttf",
    "inter-700" => "assets/fonts/inter/inter-700.ttf",
    "geist-400" => "assets/fonts/geist/geist-400.ttf",
    "geist-500" => "assets/fonts/geist/geist-500.ttf",
    "geist-600" => "assets/fonts/geist/geist-600.ttf",
    "geist-700" => "assets/fonts/geist/geist-700.ttf",
    "space-grotesk-400" => "assets/fonts/space-grotesk/space-grotesk-400.ttf",
    "space-grotesk-500" => "assets/fonts/space-grotesk/space-grotesk-500.ttf",
    "space-grotesk-600" => "assets/fonts/space-grotesk/space-grotesk-600.ttf",
    "space-grotesk-700" => "assets/fonts/space-grotesk/space-grotesk-700.ttf",
    "jetbrains-mono-400" => "assets/fonts/jetbrains-mono/jetbrains-mono-400.ttf",
    "jetbrains-mono-600" => "assets/fonts/jetbrains-mono/jetbrains-mono-600.ttf",
    "geist-mono-400" => "assets/fonts/geist-mono/geist-mono-400.ttf",
    "geist-mono-600" => "assets/fonts/geist-mono/geist-mono-600.ttf",
};

/// egui's defaults plus every bundled font, with the families pointed at `choice`.
/// egui's own fonts stay as fallbacks for symbols the bundled Latin subsets lack.
pub fn definitions(choice: Choice) -> FontDefinitions {
    let mut defs = FontDefinitions::default();
    for (key, bytes) in FONTS {
        defs.font_data
            .insert(key.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    let fallback = |family: FontFamily, defs: &FontDefinitions| {
        defs.families.get(&family).cloned().unwrap_or_default()
    };
    let text_fallback = fallback(FontFamily::Proportional, &defs);
    let mono_fallback = fallback(FontFamily::Monospace, &defs);
    let with = |first: &str, rest: &[String]| {
        std::iter::once(first.to_owned())
            .chain(rest.iter().cloned())
            .collect::<Vec<_>>()
    };

    let ([w400, w500, w600, w700], [m400, m600]) = choice.keys();
    for (name, key) in [
        ("w400", w400),
        ("w500", w500),
        ("w600", w600),
        ("w700", w700),
    ] {
        defs.families
            .insert(FontFamily::Name(name.into()), with(key, &text_fallback));
    }
    for (name, key) in [("mono400", m400), ("mono600", m600)] {
        defs.families
            .insert(FontFamily::Name(name.into()), with(key, &mono_fallback));
    }
    defs.families
        .insert(FontFamily::Proportional, with(w400, &text_fallback));
    defs.families
        .insert(FontFamily::Monospace, with(m400, &mono_fallback));
    defs
}
