# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-22

Initial release.

### Widgets

- `StyledButton` with `bg` / `hover_bg` / `active_bg` / `text_color` / `border` / `corner_radius` / `font(FontId)` / `image` / `min_width` / `min_height`.
- `StyledLabel` with `font_size` / `font(FontId)` / `bold` / `italics` / `wrap`. `RichText` accepted as an escape hatch for inline mixed styling. No pseudo-state (labels don't have hover/focus/active).
- `StyledTextEdit` with `hint` / `password` / `multiline` / `char_limit` / `font(FontId)` / `desired_width` / `horizontal_align` plus `focus_border` for focus styling.
- `StyledCheckbox` with full pseudo-state support.
- `StyledSlider` generic over `T: Numeric` with `text` and `step` builders.
- `StyledComboBox` with closure-based menu contents.
- Every interactive widget exposes `.id(impl Hash)` to override `ui.next_auto_id()` and pin pseudo-state across conditional rendering.

### Containers

- `StyledFrame` with `bg` / `border` / `padding` / `margin` / `corner_radius` / `align` / `justify`.
- `StyledRow` / `StyledColumn` with `gap` / `align` (cross-axis) / `justify` (main-axis, start/center/end). No flexbox `space-*` - immediate-mode constraint, documented.
- `StyledArea` - top-level positioned container for modals, backdrops, toasts. Operates on `&Context`, not `&mut Ui`. Supports `anchor` / `fixed_pos` / `order` / `fill_screen` plus all box-style builders.
- All container `show` methods are generic over the body's return type: `fn show<R>(self, ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>`.

### Theme system

- `StyledTheme` - geometry and typography tokens only (corner radii, spacing, font sizes, font families). No colors - those are domain-specific and don't belong in a library.
- `font_display(size)` / `font_body(size)` / `font_mono(size)` helpers compose family + size into a `FontId` at the call site.
- `WebPalette` - optional starter color struct with web/SaaS vocabulary (`accent`, `error`, `fg_on_accent`, etc.). Opt-in via `set_design_data(WebPalette { … })`.
- `DesignSlots` trait - generic typed storage on `egui::Context` for any design data (colors, audio cues, syntax themes). One `TypeId`-keyed slot per type.
- `DesignSlots::design::<T>() -> (StyledTheme, T)` single-call accessor for the theme + one user-defined type.
- `ThemeExt::set_styled_theme` / `styled_theme` as thin convenience wrappers around `DesignSlots`.

### Composition

- `Apply` trait for composable style functions: `Styled::button("Save").apply(primary_button(&theme))`.
- `Styled` namespace entry point (`Styled::button`, `Styled::frame`, `Styled::area`, …).
- `prelude` module for one-import setup.

### Color utilities

- `ColorExt` trait on `Color32`: `with_alpha`, `lerp`, `lighten`, `darken`. Naive sRGB - good enough for hover/active variants; doc points at the [`palette`](https://crates.io/crates/palette) crate for perceptually-uniform work.
- `rgb` / `rgba` shorthand constructors.

### Internals

- `SharedStyle` resolver with pseudo-state fall-through to egui's active visuals.
- `PseudoState` tracking via `egui::Memory` (one-frame lag, imperceptible).
- `impl_style_builders!` macro generates a uniform builder API across every styled type, including generic types.

### Examples

Seven runnable examples in `examples/`:

- `basic` - buttons + frame
- `containers` - row / column / nesting
- `text_edit` - focus-state styling
- `theme_demo` - live theme switching with swatches (midnight ↔ parchment)
- `composable_styles` - `Apply` + reusable style functions
- `all_widgets` - every widget in one screen
- `game_over` - full game-over screen (Area + Column + custom palette + DesignSlots + ColorExt)
