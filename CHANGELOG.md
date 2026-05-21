# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `StyledTextEdit`: `.char_limit()`, `.font(FontId)`, `.desired_width()`, `.horizontal_align()` pass-throughs to `egui::TextEdit`.
- `StyledLabel::font(FontId)` for setting font family + size in one chain (no need to detour through `RichText` just to pick a font family).
- `StyledTheme`: `font_family_display`, `font_family_body`, `font_family_mono` tokens plus `font_display(size)` / `font_body(size)` / `font_mono(size)` helpers for composing a `FontId` from the theme.
- `DesignSlots` trait — generic typed storage on `egui::Context`. `set_design_data::<T>` / `design_data::<T>` lets apps store any design data (colors, audio cues, syntax themes) without the library predicting categories.
- `WebPalette` — optional starter color struct with web/SaaS vocabulary (`accent`, `error`, `fg_on_accent`, etc.). Opt-in via `set_design_data(WebPalette { … })`.

### Changed

- **Breaking:** `StyledTheme` no longer carries colors — it holds only geometry (radii, spacing) and typography (sizes, families). Colors moved to `WebPalette` (opt-in) or user-defined structs stored via `DesignSlots`. The library no longer dictates color vocabulary.
- `ThemeExt::set_styled_theme` / `styled_theme` are now thin wrappers over `DesignSlots`. The underlying storage uses one `TypeId`-keyed slot per type.
- `StyledTextEdit::font(...)` and `StyledLabel::font(...)` override `font_size` from `SharedStyle` when both are set.
- Explicit `desired_width(...)` on `StyledTextEdit` now wins over `full_width()`.

### Removed

- Color fields (`bg_*`, `fg_*`, `accent*`, `error`, `warning`, `success`, `border*`) from `StyledTheme`. Migrate via `WebPalette` (drop-in for the same fields) or by defining a domain-specific color struct.

## [0.1.0] - 2026-05-21

Initial release.

### Added

- `SharedStyle` resolver with pseudo-state (hover / focus / active) fall-through to egui defaults.
- `PseudoState` tracking via `egui::Memory` for per-widget interaction state.
- Styled widgets: `StyledButton`, `StyledLabel`, `StyledTextEdit`, `StyledCheckbox`, `StyledSlider`, `StyledComboBox`.
- Styled containers: `StyledFrame`, `StyledRow`, `StyledColumn`.
- `StyledTheme` design tokens (colors, spacing, corner radii, typography) with neutral default.
- `ThemeExt` trait on `egui::Context` for `set_styled_theme` / `styled_theme` round-trip.
- `Apply` trait for composable style functions.
- `Styled` namespace entry point and `prelude` module.
- Six runnable examples covering every widget, the theme system, and style composition.
