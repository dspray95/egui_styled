//! Bundled CC0 photos (see `assets/images/CREDITS.md`), decoded by egui_extras' loaders.

use egui::ImageSource;

pub struct Track {
    pub title: &'static str,
    pub artist: &'static str,
    pub seconds: u32,
    pub cover: ImageSource<'static>,
}

pub fn tracks() -> [Track; 6] {
    [
        Track {
            title: "Neon Streets",
            artist: "Night Shift",
            seconds: 214,
            cover: egui::include_image!("assets/images/cover_neon.jpg"),
        },
        Track {
            title: "Tidal",
            artist: "Low Coast",
            seconds: 187,
            cover: egui::include_image!("assets/images/cover_ocean.jpg"),
        },
        Track {
            title: "Alpenglow",
            artist: "Summit",
            seconds: 241,
            cover: egui::include_image!("assets/images/cover_peak.jpg"),
        },
        Track {
            title: "Through the Pines",
            artist: "Hollow Oak",
            seconds: 198,
            cover: egui::include_image!("assets/images/cover_forest.jpg"),
        },
        Track {
            title: "Dune Sea",
            artist: "Mirage",
            seconds: 226,
            cover: egui::include_image!("assets/images/cover_dunes.jpg"),
        },
        Track {
            title: "Northern Lights",
            artist: "Polar",
            seconds: 263,
            cover: egui::include_image!("assets/images/cover_aurora.jpg"),
        },
    ]
}

pub fn sunset() -> ImageSource<'static> {
    egui::include_image!("assets/images/backdrop_sunset.jpg")
}
