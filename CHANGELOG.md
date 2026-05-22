# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `DesignSlots::design::<T>() -> (StyledTheme, T)` — single-call accessor for the theme + one user-defined type. Replaces the two-line `let theme = ...; let colors = ...;` ceremony at the top of every panel.
- `ColorExt`: `lerp(other, t)`, `lighten(factor)`, `darken(factor)` — naive sRGB interpolation. Good enough for hover/active variants; doc points at the [`palette`](https://crates.io/crates/palette) crate for perceptually-uniform color work.
- `.id(impl Hash)` builder on `StyledButton`, `StyledTextEdit`, `StyledCheckbox`, `StyledSlider`. Overrides `ui.next_auto_id()` for pseudo-state lookup so hover/focus/active don't get misattributed when conditional rendering shifts widget positions.
- `StyledTextEdit`: `.char_limit()`, `.font(FontId)`, `.desired_width()`, `.horizontal_align()` pass-throughs to `egui::TextEdit`.
- `StyledLabel::font(FontId)` for setting font family + size in one chain (no need to detour through `RichText` just to pick a font family).
- `StyledButton::font(FontId)` — same pattern as label / text_edit. Closes the symmetry gap across all three text-bearing widgets.
- `min_height(f32)` and `max_height(f32)` builders on every styled type (the underlying `SharedStyle` fields existed but weren't exposed). `StyledButton::min_height` controls intrinsic button height directly, matching `min_width`.
- `align(Align)` builder on `StyledColumn`, `StyledRow`, `StyledFrame`, and `StyledArea`. Column uses `Layout::top_down`, row uses `Layout::left_to_right`, frame wraps in `with_layout` only when set. Replaces ad-hoc `ui.vertical_centered` wrappers.
- `justify(Align)` builder on the same four containers — main-axis distribution via `Layout::with_main_align`. Covers `start`/`center`/`end`. **Note:** does *not* implement flexbox's `space-between`/`space-around`/`space-evenly` — those require two-pass layout or one-frame-lag caching in immediate-mode, neither of which fit the current API cleanly.
- `margin_left(f32)` and `margin_right(f32)` builders alongside the existing `margin_top` / `margin_bottom`, for callers that want non-vertical insets.

### Changed

- **Breaking (minor):** `margin_top` / `margin_bottom` now take `f32` instead of `i8`. Matches `theme.spacing_*` (also `f32`), so call sites read as `.margin_top(theme.spacing_md)` rather than `.margin_top(theme.spacing_md as i8)`. Internally clamps to egui's `i8` `Margin` range. Existing call sites passing integer literals (`margin_top(8)`) compile fine after switching to `8.0`.
- `Styled::area()` — top-level positioned container (modals, backdrops, toasts). Operates on `&Context` rather than `&mut Ui`. Supports `anchor`, `fixed_pos`, `order`, `fill_screen`, plus every `SharedStyle` box builder.
- `ColorExt::with_alpha(u8)` trait method on `Color32`. One-chain alpha replacement; no more `to_array` / `from_rgba_unmultiplied` dance. Re-exported from the prelude.

### Changed

- **Breaking:** `StyledColumn::show`, `StyledRow::show`, `StyledFrame::show` are now generic over the body's return type — `fn show<R>(self, ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>`. Existing call sites that returned `()` still type-check unchanged; new sites can return values without hoisting `let mut x: Option<…>` outside.
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
