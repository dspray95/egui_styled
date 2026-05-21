# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
