# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `StyledArea::fixed_pos_centered(Pos2)` — places the area so its *center* lands at the given screen-space point, using last-frame's measured size cached in `egui::Memory`. For diegetic UI anchored to world-space points (HP bars, damage numbers). One-frame pop on first appearance; works cleanly for stable-size content.
- `StyledButton` now reads `SharedStyle::padding`, wiring it through to `ui.spacing_mut().button_padding`. Closes the silent no-op where `.padding(...)` on a button used to do nothing. **Symmetric only** — egui's `Button` doesn't support per-side padding; asymmetric `Margin` collapses to `max(left, right)` / `max(top, bottom)`. For true asymmetric, wrap in a `Styled::frame` with matching `bg`.
- `Shadow` decoration type + `.shadow(offset, width, color)` / `.shadow_filled(offset, color)` builders on every styled widget. Paints offset copies of the widget rect on the same layer, underneath the widget. Multiple `.shadow()` calls append — use two for a chromatic-aberration glitch look, one for a conventional drop shadow. Shadows inherit the widget's `corner_radius`. No layout reflow (CSS `box-shadow` semantics). Wired into `StyledButton` and `StyledFrame`; see `examples/shadows.rs`.
- `.accent(Color32)` / `.hover_accent(Color32)` builders on all styled widgets — maps to egui's `selection.bg_fill` / `selection.stroke` (slider trailing fill, text-selection highlight, focused text-edit border).
- `SharedStyle::resolve_per_state` — resolves a `PerStateStyle` with independent `ResolvedStyle` for each of `inactive` / `hovered` / `active` / `focused`. Used internally to write per-state colours into the matching `WidgetVisuals` slot so egui's own hover/active response picks the right variant.
- `SharedStyle::apply_to_visuals` — central helper that handles all egui visuals quirks: syncs `weak_bg_fill` (ComboBox button background), zeros `expansion` (prevents border rect drift), sets `selection.bg_fill` and `selection.stroke`, sets `extreme_bg_color` (TextEdit background). All widget `show()` implementations now delegate to this instead of hand-rolling overrides.
- `StyledComboBox` now tracks `PseudoState` (was missing entirely — `hover_bg`, `active_bg`, `focus_bg`, `hover_border`, `focus_border`, `hover_text_color` were silently dropped regardless of interaction). Also added `font_size`, `padding`, `margin`, `cursor`, and `shadows` support.
- `StyledCheckbox` now honours `padding`, `font_size` (applied to the label), `cursor`, and `shadows`.
- `StyledSlider` now honours `padding`, `cursor`, `shadows`, `min_width`, and `min_height`. Trailing fill colour is now controllable via `.accent(...)`.
- `StyledLabel` now renders a real `Frame` so `.bg()`, `.border()`, `.corner_radius()`, `.padding()`, and `.shadows()` all take effect. Also added `cursor` support.
- `StyledTextEdit` now honours `cursor` and `shadows`. Border and padding fixes applied (see **Fixed**).

### Fixed

- `StyledTextEdit`: `.padding(...)` was accepted by the builder but never applied. Now wired through to both `TextEdit::margin()` (text layout) and a custom `Frame::inner_margin` (visual paint). Previously the default `Margin::symmetric(4, 2)` was always used regardless of what the caller set.
- `StyledTextEdit`: border always rendered as 1 px white regardless of `.border()`/`.focus_border()` settings. Root cause: egui's `TextEdit` re-derives its border from `visuals.selection.stroke` when focused and from `widgets.style(response).bg_stroke` when not, then expands the inner margin by `expansion − stroke.width`. The fix passes a fully-built custom `Frame` to `TextEdit::frame(...)`, taking egui's `custom_frame = true` branch that skips the visuals-override path entirely, and also sets `selection.stroke` for the focused state.
- `StyledComboBox`: `.bg()` / `.hover_bg()` / `.active_bg()` had no effect on the closed button. egui's ComboBox button paints with `visuals.weak_bg_fill`, not `bg_fill`. The `apply_to_visuals` helper now keeps both in sync.
- `StyledComboBox`: border was rendered on an expanded rect due to `visuals.expansion` not being zeroed. Fixed by `apply_to_visuals` zeroing `expansion` on all widget states.
- `StyledSlider`: slider trailing fill (the filled/progress portion of the rail) was uncontrollable — it reads `visuals.selection.bg_fill` which was never overridden. Now set via `.accent(...)`.
- All widgets: per-state colour variants (`hover_bg`, `active_bg`, `focus_bg`, etc.) were previously resolved for the *current* pseudo-state and then written uniformly to all three `WidgetVisuals` slots (`inactive` / `hovered` / `active`). This collapsed differentiation: the colour correct for the current frame's state was applied everywhere, so on the very next frame when egui's own response state differed from our stored `PseudoState`, the wrong variant was shown. `resolve_per_state` fixes this by resolving each variant independently and writing it into the matching slot.

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
