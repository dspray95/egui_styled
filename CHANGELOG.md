# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-06-10

### Added

- **`StyledSpacer`** (`Styled::spacer()`) - a flexible spacer that consumes all remaining main-axis space in the current layout, pushing following siblings to the far edge. Only meaningful inside a bounded container (`full_width` / `full_height` / fixed). Works for both horizontal (inside `StyledRow`) and vertical (inside `StyledColumn`) axes.

- **Percentage sizing** - `width_pct(pct: f32)` and `height_pct(pct: f32)` builders on every styled type. Resolve to a definite size as a percentage of the parent's available width/height at render time, superseding `full_width` / `full_height`. Composed with `min_width` / `max_width` / `min_height` / `max_height` as clamps after resolution.

- **`WrappingRow`** (`Styled::row().wrap()`) - a horizontal row whose children flow onto new lines when they run out of width (CSS `flex-wrap: wrap`). Uses a cross-frame invisible-measurement pass to record each item's natural width, then places items on lines manually. This works correctly with `Styled::button`, `Styled::label`, and other scope-isolated styled widgets that egui's native `with_main_wrap` can't wrap. Built via an item API: `.item(|ui| { ... })` chains return `WrappingRow`; `.show(ui)` renders.

- **Distribution** - three CSS justify-content modes on `StyledRow`, each returning a `DistributedRow` with the same `.item(fn).show(ui)` API:
  - `space_between()` - items pinned to both ends, equal gaps between items.
  - `space_around()` - equal space on each side of each item (half-gap at edges).
  - `space_evenly()` - equal space before, between, and after every item (same gap everywhere).
  `DistributedRow` measures item widths cross-frame (invisible first pass), computes slack, and places items with `allocate_ui_with_layout`. Inherits `gap` as a minimum gap floor and all box styling from the originating `StyledRow`.

- **Aspect ratio** - `aspect_ratio(ratio: f32)` builder on every styled type (width ÷ height, CSS convention: `16.0/9.0` is a wide box). Derives height from a resolved definite width (`width_pct` or `full_width`); no-op when no definite width is available. Height precedence: `fill_size` > `height_pct` > **aspect-derived** > `full_height`. Clamped by `min_height` / `max_height`.

- **`hover_text_color` builder** on every styled type. Previously `SharedStyle` had the field but the builder was never added to the macro, so it was impossible to set without constructing `SharedStyle` directly.

- **Centralized size resolution** via `SharedStyle::resolve_size` — one method encodes the full sizing precedence for all widgets and containers. All leaf widgets (`StyledButton`, `StyledLabel`, `StyledSlider`, `StyledCheckbox`, `StyledComboBox`, `StyledTextEdit`, `StyledImage`) now delegate through it, closing the class of bug where a new `SharedStyle` field works on frames but is silently ignored on interactive widgets.

- **`max_width` and `max_height` now honored on all leaf widgets.** `StyledButton`, `StyledLabel`, `StyledSlider`, `StyledCheckbox` previously accepted these builders but applied them only on containers. They now pin a hard upper bound on the widget's allocated size.

- **`aspect_ratio` now honored on all interactive widgets.** Previously `aspect_ratio` only applied to `StyledFrame` (and containers). It now works on every styled widget via the centralized resolver.

- **Builder coverage compile-test** in `style_builders.rs`. Running `cargo test --lib` will fail at compile time if a `SharedStyle` field is added without a corresponding builder method in the macro.

### Fixed

- **`aspect_ratio` was silently ignored on `StyledImage`.** The image widget had a bespoke size-resolution block that never called `resolved_aspect_height`. It now delegates to `resolve_size` like every other leaf widget.

- **`full_width` / `width_pct` in a floating `StyledArea` (without `fill_screen`) no longer sets `min_width` to infinity.** Non-fill-screen areas expose an unbounded available width to their children; percentage and full-width sizing now degrades gracefully to content-sized in that context.

### Changed

- **`StyledRow::wrap()`** now returns `WrappingRow` instead of `StyledRow` with a `wrap: bool` flag. The new `WrappingRow` requires the item-based API (`.item(fn).show(ui)`); the old closure-based `.show(ui, |ui| { ... })` is no longer available on wrapping rows. Plain (non-wrapping) `StyledRow::show` is unchanged.
- **`StyledRow::space_between()` / `space_around()` / `space_evenly()`** return `DistributedRow` instead of `StyledRow`. These previously existed as layout hints and did nothing; they are now fully implemented.

## [0.5.5] - 2026-06-05

### Added

- **CSS-style per-side borders** on every styled type. New builders `border_top` / `border_right` / `border_bottom` / `border_left`, plus the convenience pairs `border_x` (left + right) and `border_y` (top + bottom), and matching `hover_border_*` / `focus_border_*` variants. Each unset side falls back to the uniform `border` for the same state (and then egui's default), following the same `focus > hover > base` precedence as the existing border. Works on `StyledFrame` and the containers that delegate to it (`StyledColumn`, `StyledRow`, `StyledStack`) as well as the interactive widgets (`StyledButton`, `StyledCheckbox`, `StyledComboBox`, `StyledImage`, `StyledLabel`, `StyledSlider`, `StyledTextEdit`).

  The uniform-border path is unchanged: per-side painting only activates when a side override is actually set, so existing borders (including their corner-radius rounding) render exactly as before. Note that partial borders are drawn as straight edges and do **not** follow `corner_radius` rounding - egui has no per-side rounded-stroke primitive.

## [0.5.4] - 2026-06-04

### Fixed

- **`StyledColumn` now delegates `align` / `justify` / `gap` to its `StyledFrame`** when it has box styling, instead of running its own `with_layout(...).with_main_align(...)` (the no-op the frame's vertical spacer replaces). Previously `Styled::column().bg(c).full_height().justify(Center)` never centered vertically because the column bypassed the spacer added in 0.5.3. `StyledRow` keeps its own `left_to_right` layout - a row's main axis is horizontal, which the (top-down) frame spacer doesn't apply to - and continues to delegate only box styling.

## [0.5.3] - 2026-06-04

### Added

- **`full_height()` builder** on every styled container. Mirrors `full_width` - stretches the container to fill the parent's available height. Available on `StyledFrame`, `StyledRow`, `StyledColumn`, `StyledArea`, and `StyledStack`.

- **`min_width` / `max_width` / `min_height` / `max_height` now applied by container frames.** The builders already existed on `SharedStyle` and worked on leaf widgets (button, label, slider) but were silently ignored by `StyledFrame` and the containers that use it (`StyledRow`, `StyledColumn`, `StyledArea`). They are now applied inside the layout closure so `.max_width(200.0)` on a frame actually caps it. `max_*` is applied before `full_width` / `full_height` so those read the already-capped available size; `min_*` is applied last so explicit minimums always win.

- **Vertical justify (`Center` / `Max`) now works on full-height and fill containers.** `.justify(Align::Center)` or `.justify(Align::Max)` on a `StyledFrame` (or any container) with a determinate height (`full_height()` set, or `fill_size` from `fill_screen()`) now vertically centers or bottom-aligns the body content. Previously `justify` was applied via egui's `with_main_align` which is a no-op for top-down layouts, so content always pinned to the top. The fix uses a measured top-spacer (same invisible-first-frame approach `StyledArea` already used) extracted into a shared `justify_body_vertically` helper. `StyledArea` now delegates to the same path via `StyledFrame`, removing ~35 lines of duplicated logic.

## [0.5.2] - 2026-06-04

### Fixed

- **`bg` fill now paints immediately while a `background_image` texture is loading.** Previously, when both `.bg(color)` and `.background_image(...)` were set, the fill was withheld until the texture finished decoding - the app's clear color showed through during the load window. The fill now paints from the first frame; the image appears on top once ready. Fade paths (`background_image_fade_in`, `reveal_with_background_image`) and the no-image path are unchanged.

## [0.5.1] - 2026-06-04

### Added

- **`background_image_fade_in(seconds: f32)`** builder on every box container. When set, the background image fades up from the `bg` backdrop over the given duration (linear alpha) the first time its texture finishes loading, instead of snapping in. The fade clock is stamped in `egui::Memory` on the first `Ready` frame, `ctx.request_repaint()` is called until alpha reaches 1.0, and the alpha is folded into `background_image_tint` (default `WHITE`). `None` / unset = current snap-in behavior unchanged. New field `SharedStyle::background_image_fade_in: Option<f32>`.

### Fixed

- **`StyledFrame::fill_size` / `StyledArea::fill_screen` no longer loses its size on window shrink.** The area was pinned with `set_min_size` only (a floor), so shrinking the window left the frame at its previous wider extent; content centered against that stale width instead of the new one. The fix pins with both `set_min_size` and `set_max_size` so the area contracts as well as expands.

### Documentation

- **Fixed broken intra-doc link** to `BackgroundImageFit::Stretch` on the `background_image_fit` builders (it rendered as plain text on docs.rs because the macro-expanded doc comment couldn't resolve the unqualified path; now uses `crate::BackgroundImageFit::Stretch`).
- **Type-level docs added** for `StyledButton` and `StyledFrame`, so their docs.rs item pages carry a summary like the other core types.
- **Shared builder methods documented** once in the `impl_style_builders!` macro body - `bg` / `hover_bg` / `active_bg` / `focus_bg`, `hover_accent`, `text_color`, `font_size`, `border` / `hover_border` / `focus_border`, `corner_radius`, `padding`, the `margin_*` setters, the sizing builders (`full_width`, `min/max_width`, `min/max_height`), `cursor`, and `visible`. This documents those methods across every styled type at once (missing-docs warnings dropped from 441 to 163).

## [0.5.0] - 2026-06-04

### Added

- **`.gap(f32)` on `StyledFrame` and `StyledArea`.** Sets `item_spacing` (both axes) on the inner `Ui`, matching the existing `.gap()` already on `StyledRow` and `StyledColumn`. Use `Styled::frame().gap(12.0)` or `Styled::area().gap(8.0)` instead of manually calling `ui.add_space(...)` between children.

- **First-class image / texture support.** egui_styled now presents images through the same styled API as boxes and buttons - themed corner radius, border, shadow, padding, and tint - without ever touching texture loading. The consuming app installs loaders (`egui_extras::install_image_loaders`) or registers native textures (`ctx.load_texture`); egui_styled only paints.

  - **`background_image` on every box container** (`StyledArea`, `StyledFrame`, `StyledRow`, `StyledColumn`, `StyledStack`). Three new `SharedStyle` builders available everywhere:
    - `.background_image(impl Into<Image<'static>>)` - accepts a finished `egui::Image`, an `ImageSource`, or an `include_image!(...)` result.
    - `.background_image_fit(BackgroundImageFit)` - `Stretch` (default, maps full texture over the rect) or `Cover` (scale-to-cover crop, preserves aspect ratio).
    - `.background_image_tint(Color32)` - multiply the texture colour before painting (default `WHITE` = no tint).
    - When both `.bg(color)` and `.background_image(...)` are set, the fill paints first and the texture on top - the fill is visible while the texture is loading.
    - The texture is drawn via `epaint::RectShape::with_texture` (same path egui's own `Frame` uses), so the tessellator handles rounded-corner clipping - no manual scissor rect needed.

  - **`StyledImage` - new inline widget** for icons, portraits, and thumbnails that flows in layout. Same shared builders as every other widget plus image-specific ones:
    - `.tint(Color32)` / `.hover_tint(Color32)` - base and hover multiply tint.
    - `.size(Vec2)` - render at exactly this size.
    - `.max_size(Vec2)` - constrain within a bounding box while maintaining aspect ratio.
    - `.fit_to_fraction(Vec2)` / `.fit_to_original(scale)` - additional fit modes.
    - `.id(impl Hash)` - stable pseudo-state across conditional rendering (same as `StyledButton::id`).
    - `Styled::image(impl Into<Image<'static>>)` entry point added to the `Styled` namespace.

  - **`BackgroundImageFit`** enum exported from the crate root and `prelude`.

- **`examples/images.rs`** - self-contained runnable showcase: `StyledImage` with corner radius / border / shadow / hover tint / circle clip, and `background_image` on a frame with stretch fill, tinted semi-transparent overlay, and `Cover` crop fit. Builds and runs with no external image files.

### Changed

- **Breaking:** `SharedStyle` gained three new public fields (`background_image`, `background_image_fit`, `background_image_tint`). Code that constructs `SharedStyle { .. }` with a full struct literal must add the new fields or use `..SharedStyle::default()`. The builder APIs and all widget `show()` implementations are unaffected.
- **egui bumped to 0.34.3** (patch release). `eframe` and `egui_kittest` dev-dependencies updated to match.
- **`examples/images.rs` migrated to `eframe::App::ui`.** The example now implements `eframe::App` (required `fn ui` method) instead of the deprecated `eframe::run_ui_native` + `CentralPanel::show` pattern. No visual change.

### Fixed

- **`.justify(Align)` now works for vertical distribution on a `fill_screen` `StyledArea`.** A `Styled::area().fill_screen().align(Center).justify(Center)` centered its content horizontally but pinned it to the top vertically (likewise `justify(Max)` did not bottom-align). Root cause: egui's `Layout::top_down(..).with_main_align(..)` cannot center/bottom-align on the main (vertical) axis - `next_frame_ignore_wrap` always aligns the frame to `Align::TOP` for top-down layouts, so `with_main_align` only ever affects the cross axis. Expanding the available rect (as the `fill_screen` size fix does) has no effect on this. The fix offsets the content with a top spacer of `(screen_height - content_height) * justify.to_factor()`, computed from the content's measured height. Because the height isn't known until the content is laid out, the first frame an area appears it renders the content *invisibly* purely to measure it and requests an immediate repaint; the content then appears already correctly positioned on the next frame, so there is no visible one-frame pop. `justify(Min)` (top, the previous behavior), `Center`, and `Max` (bottom) all now resolve correctly. Only applies when both `fill_screen()` and `justify(..)` are set; all other container paths are unchanged. A regression test asserts the content is hidden on the first frame and visible and vertically centered (within 2px of screen center) on the second.

- **`StyledArea::fill_screen()` now fills the screen.** The area reserved the full viewport but the inner `egui::Frame` still shrink-wrapped to its content, so a `fill_screen` area with a `background_image` rendered as a small box in the top-left corner instead of covering the viewport. Root cause: `ui.set_min_size(screen_size)` was called on the area's outer `Ui` rather than on the frame's inner `Ui`; `egui::Frame::show` allocates a fresh child `Ui` that does not inherit the parent's min-size. The fix passes the min-size down into the frame body, so the frame's own `response.rect` (the rect used to paint the background image) matches the full content rect. The positioning logic (`fixed_pos(ctx.content_rect().min)`) and the priority chain (`fixed_pos_centered > fill_screen > fixed_pos > anchor`) are unchanged. A regression test asserts the invariant: a `fill_screen` area with a single-label body produces a rect within 1px of `ctx.content_rect().size()`.

## [0.4.0] - 2026-06-03

### Added

- **Declarative text effects on `StyledLabel`.** Four glyph-level appearance primitives that stamp the laid-out galley at offsets/colors rather than painting offset rectangles like the box-shadow `.shadow()`. All are *static* - they take a plain per-frame value; animation (intensity curves, scale punch, animated offsets) stays consumer-side via `ctx.data` + `request_repaint`. The library never animates. Effects compose freely on a single label.
  - `.text_shadow(offset, color)` - repaint the glyph run at `offset` in `color`. Multiple calls compose, so a chromatic-aberration split is `.text_shadow(vec2(-2, 0), cyan).text_shadow(vec2(2, 0), magenta)` - no dedicated chromatic feature needed.
  - `.outline(width, color)` - a faux stroke from 8 compass-direction glyph stamps at `width` pixels.
  - `.glow(color, radius, intensity)` - a soft halo that follows the letterforms. egui has no blur pass, so glow is approximated by stamping the text on a sunflower (Vogel) disk - an aperiodic golden-angle spiral (no grid/spokes) of faint copies weighted by a window that reaches zero at the edge, so overlapping copies blend into a smooth halo without moiré, ghosts, or popping when `intensity` animates. `.glow_quality(samples)` tunes the base stamp density (default 64); the real count scales with radius² so large glows stay smooth, and brightness is independent of both radius and quality. This is the priciest primitive - drop `samples` for dense UIs.
  - `.scale(factor, pivot)` - scale the painted glyphs about an `Align2` pivot via `TSTransform` over the label's shape range (the same mechanism `Styled::stack()` uses to translate). The allocated layout footprint stays at natural size, so siblings don't shift; pair with `Styled::stack().layer_fixed(...)` to control overflow.
  - **`StyledTheme` shadow & glow scale tokens** - `shadow_sm`/`shadow_md`/`shadow_lg` (offset `Vec2`s, a downward drop ramp) and `glow_sm`/`glow_md`/`glow_lg` (radius `f32`s), following the existing sm/md/lg ramp. The effect methods take raw `Vec2`/`f32`, so pass a token (`theme.glow_md`) or an animated value interchangeably.
- **Snapshot / visual-regression tests** via `egui_kittest` (new dev-dependency). `tests/text_effects.rs` covers shadow, chromatic aberration, outline, glow, scale, and a composed case; baselines live in `tests/snapshots/`.
- **`examples/text_effects.rs`** - animated showcase of all four primitives plus composed effects, demonstrating the consumer-side animation model. `examples/game_over.rs` migrated to express its chromatic `[ENTER]` glitch as a single `.text_shadow()`-pair label and adds a glow to the score.

### Changed

- **Breaking:** `StyledTheme` gained six public fields (`shadow_sm/md/lg`, `glow_sm/md/lg`). Code that constructs `StyledTheme { .. }` with a full struct literal must add the new fields or use `..StyledTheme::default()`. Reading the theme and the builder APIs are unaffected.

## [0.3.0] - 2026-05-29

### Added

- `StyledStack::layer_fixed(size, align, fn)` - the size-decoupling counterpart to `layer_offset`'s position-decoupling. Declares an explicit layout footprint: only `size` is contributed to the stack's union allocation, while the layer's actual content is positioned within that box via `align` and is free to overflow it visually. Designed for scale-punch / bounce / pop animations on fixed-layout elements - a hero number that briefly renders at 1.4× no longer pushes siblings. `Align2::CENTER_CENTER` gives symmetric overflow (scale punch); corner aligns give badge-style overflow. Note: overflowing content paints outside the allocated rect and can draw over siblings in the parent flow; this is intentional.
- `StyledStack` overlay container (`Styled::stack()`) - renders all children at a shared origin instead of in flow, the one container that can put multiple things in the same pixels. `.layer(fn)` / `.layer_offset(offset, fn)` anchor at the shared origin (optionally nudged by a pixel offset, e.g. for chromatic-aberration glitch effects); `.layer_aligned(Align2, fn)` positions a layer within the union of all *preceding* layers (so "background first, overlay centered on it" works). Z-order is call order (first = bottom). `.sense(Sense)` makes the whole stack interactive (defaults to hover). Because layers paint before the parent decides the stack's final position (especially under centering, where `next_widget_position` can be infinite), the stack paints at a provisional origin then translates only its own shapes (`PaintList::transform_range`) into the allocated rect - no extra layer, so z-order stays correct and multiple stacks in one `Ui` don't collide. Caveat: interactive widgets inside a stack are hit-tested at the pre-translation position. See `examples/stack.rs` and the updated `examples/game_over.rs`.

### Fixed

- `margin` was silently dropped on frameless containers. Setting only `.margin*(...)` on a `Styled::row()`, `Styled::column()`, or `Styled::stack()` (with no other frame styles) had no effect - margin is applied via the wrapper frame's `outer_margin`, but `SharedStyle::has_frame_styles()` excluded margin, so a margin-only container took the bare-render branch and skipped the frame entirely. The same `.margin_top()` worked on a `StyledLabel` (which always wraps itself in a frame), making the inconsistency surprising. `has_frame_styles()` now counts `margin`, so a margin-only container routes through `StyledFrame` (a transparent, zero-padding frame whose only effect is the outer margin) and the spacing is honored consistently.

### Changed

- **Breaking:** `StyledLabel::wrap(bool)` replaced with explicit wrap modes. `StyledLabel` now exposes egui's full `TextWrapMode` via `.wrap_mode(TextWrapMode)`, plus `.wrap()` / `.truncate()` / `.extend()` shortcuts. The old `.wrap(true)` / `.wrap(false)` collapsed three egui modes into a bool and could not reach `Extend` ("lay out at natural width, never wrap or truncate") - the mode needed to keep text intact inside tight rows and stacks. Migration: `.wrap(true)` → `.wrap()`, `.wrap(false)` → `.truncate()`. `TextWrapMode` is re-exported from the prelude.

## [0.2.2] - 2026-05-28

### Fixed

- `.visible(false)` leaked invisibility to sibling widgets. `Ui::set_invisible()` mutates the painter/enabled state of the `Ui` it's called on, so calling it on the shared parent made *every* widget drawn afterwards in the same `Ui` invisible too - a hidden widget mid-layout (e.g. in a column) wiped out all following siblings. Now each widget renders inside a child scope (`Ui::scope`) and calls `set_invisible()` there, containing the effect to itself while still reserving layout space. Applies to `StyledLabel`, `StyledButton`, `StyledTextEdit`, `StyledSlider`, `StyledComboBox`, and `StyledCheckbox`. See `examples/visibility.rs`.

### Fixed

- `.visible(false)` was honored only by `StyledLabel`; calling it on any other widget or container silently did nothing. Now wired through every `show()` via `Ui::set_invisible()` - reserves layout space, paints nothing, implies disabled (no hover/click interaction). Applies to `StyledButton`, `StyledTextEdit`, `StyledCheckbox`, `StyledSlider`, `StyledComboBox`, `StyledFrame`, `StyledRow`, `StyledColumn`, and `StyledArea`. Children of invisible containers inherit invisibility automatically.
- `StyledLabel::visible(false)` implementation simplified from a bespoke `allocate_exact_size` branch to the same `set_invisible()` path used by all other widgets.

## [0.2.0] - 2026-05-27

### Added

- `StyledArea::fixed_pos_centered(Pos2)` - places the area so its *center* lands at the given screen-space point, using last-frame's measured size cached in `egui::Memory`. For diegetic UI anchored to world-space points (HP bars, damage numbers). One-frame pop on first appearance; works cleanly for stable-size content.
- `StyledButton` now reads `SharedStyle::padding`, wiring it through to `ui.spacing_mut().button_padding`. Closes the silent no-op where `.padding(...)` on a button used to do nothing. **Symmetric only** - egui's `Button` doesn't support per-side padding; asymmetric `Margin` collapses to `max(left, right)` / `max(top, bottom)`. For true asymmetric, wrap in a `Styled::frame` with matching `bg`.
- `Shadow` decoration type + `.shadow(offset, width, color)` / `.shadow_filled(offset, color)` builders on every styled widget. Paints offset copies of the widget rect on the same layer, underneath the widget. Multiple `.shadow()` calls append - use two for a chromatic-aberration glitch look, one for a conventional drop shadow. Shadows inherit the widget's `corner_radius`. No layout reflow (CSS `box-shadow` semantics). Wired into `StyledButton` and `StyledFrame`; see `examples/shadows.rs`.
- `.accent(Color32)` / `.hover_accent(Color32)` builders on all styled widgets - maps to egui's `selection.bg_fill` / `selection.stroke` (slider trailing fill, text-selection highlight, focused text-edit border).
- `SharedStyle::resolve_per_state` - resolves a `PerStateStyle` with independent `ResolvedStyle` for each of `inactive` / `hovered` / `active` / `focused`. Used internally to write per-state colours into the matching `WidgetVisuals` slot so egui's own hover/active response picks the right variant.
- `SharedStyle::apply_to_visuals` - central helper that handles all egui visuals quirks: syncs `weak_bg_fill` (ComboBox button background), zeros `expansion` (prevents border rect drift), sets `selection.bg_fill` and `selection.stroke`, sets `extreme_bg_color` (TextEdit background). All widget `show()` implementations now delegate to this instead of hand-rolling overrides.
- `StyledComboBox` now tracks `PseudoState` (was missing entirely - `hover_bg`, `active_bg`, `focus_bg`, `hover_border`, `focus_border`, `hover_text_color` were silently dropped regardless of interaction). Also added `font_size`, `padding`, `margin`, `cursor`, and `shadows` support.
- `StyledCheckbox` now honours `padding`, `font_size` (applied to the label), `cursor`, and `shadows`.
- `StyledSlider` now honours `padding`, `cursor`, `shadows`, `min_width`, and `min_height`. Trailing fill colour is now controllable via `.accent(...)`.
- `StyledLabel` now renders a real `Frame` so `.bg()`, `.border()`, `.corner_radius()`, `.padding()`, and `.shadows()` all take effect. Also added `cursor` support.
- `StyledLabel` now honours `.min_height(f32)` - reserves a fixed vertical slot every frame so surrounding layout doesn't reflow when content changes.
- `.visible(bool)` builder on all styled widgets (via `SharedStyle`) - when `false`, the widget allocates no space but skips painting. Pair with `.min_height()` on `StyledLabel` to reserve a stable slot for blinking / conditionally-shown text.
- `StyledTextEdit` now honours `cursor` and `shadows`. Border and padding fixes applied (see **Fixed**).

### Fixed

- `StyledTextEdit`: `.padding(...)` was accepted by the builder but never applied. Now wired through to both `TextEdit::margin()` (text layout) and a custom `Frame::inner_margin` (visual paint). Previously the default `Margin::symmetric(4, 2)` was always used regardless of what the caller set.
- `StyledTextEdit`: border always rendered as 1 px white regardless of `.border()`/`.focus_border()` settings. Root cause: egui's `TextEdit` re-derives its border from `visuals.selection.stroke` when focused and from `widgets.style(response).bg_stroke` when not, then expands the inner margin by `expansion − stroke.width`. The fix passes a fully-built custom `Frame` to `TextEdit::frame(...)`, taking egui's `custom_frame = true` branch that skips the visuals-override path entirely, and also sets `selection.stroke` for the focused state.
- `StyledComboBox`: `.bg()` / `.hover_bg()` / `.active_bg()` had no effect on the closed button. egui's ComboBox button paints with `visuals.weak_bg_fill`, not `bg_fill`. The `apply_to_visuals` helper now keeps both in sync.
- `StyledComboBox`: border was rendered on an expanded rect due to `visuals.expansion` not being zeroed. Fixed by `apply_to_visuals` zeroing `expansion` on all widget states.
- `StyledSlider`: slider trailing fill (the filled/progress portion of the rail) was uncontrollable - it reads `visuals.selection.bg_fill` which was never overridden. Now set via `.accent(...)`.
- `StyledLabel`: when `.visible(false)`, the hidden frame allocated `vec2(0, 0)` causing width to collapse to zero on hidden frames and full-width on visible ones, producing layout shift. Now allocates `available_width` so the slot is stable in both axes.
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
