# egui_styled

A styling layer for [egui](https://github.com/emilk/egui) that adds per-widget hover/focus/active styling, semantic design tokens, and composable style functions. Tailwind-style utility styling for an immediate-mode toolkit, with Flutter-shaped widgets.

Build UI thinking about *what* things look like, rather than how egui's internals work.

## Why?

egui's styling lives on `ui.visuals_mut()`, which is global to the current `Ui`. Applying a custom hover color to *one* button means cloning the visuals, mutating three `WidgetVisuals` states, and wrapping in `ui.scope` every time on every widget. The pattern is correct but is tricky to scale.

`egui_styled` wraps egui's widgets with builder APIs that handle the visuals dance for you, and adds a small theme system so colors/spacing/radii live in one place.

## Before / After

A card with two themed text inputs and two buttons.

### With `egui_styled`

```rust
use egui_styled::prelude::*;

// One line fetches both. `WebPalette` is the opt-in starter color set;
// swap for your own struct (game palette, IDE syntax theme, etc).
let (t, c) = ui.ctx().design::<WebPalette>();

Styled::frame()
    .bg(c.bg_surface)
    .corner_radius(t.rounding_lg)
    .padding(t.spacing_lg)
    .border(1.0, c.border)
    .show(ui, |ui| {
        Styled::text_edit(&mut username)
            .hint("Username")
            .full_width()
            .bg(c.bg_secondary)
            .corner_radius(t.rounding_md)
            .border(1.0, c.border)
            .focus_border(1.0, c.border_focus)
            .show(ui);
        Styled::text_edit(&mut email)
            .hint("Email")
            .full_width()
            .bg(c.bg_secondary)
            .corner_radius(t.rounding_md)
            .border(1.0, c.border)
            .focus_border(1.0, c.border_focus)
            .show(ui);
        Styled::row().gap(t.spacing_sm).show(ui, |ui| {
            Styled::button("Cancel")
                .bg(Color32::TRANSPARENT)
                .hover_bg(c.bg_elevated)
                .show(ui);
            Styled::button("Save")
                .bg(c.accent)
                .hover_bg(c.accent_hover)
                .text_color(c.fg_on_accent)
                .show(ui);
        });
    });
```

<details>
<summary><b>… and the same UI in raw egui (click to expand the ~50 lines of <code>visuals_mut</code> copy-paste)</b></summary>

```rust
ui.scope(|ui| {
    egui::Frame::none()
        .fill(rgb(30, 30, 30))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(16.0))
        .stroke(Stroke::new(1.0, rgb(60, 60, 60)))
        .show(ui, |ui| {
            // Username copy 1 of 2
            ui.scope(|ui| {
                let v = ui.visuals_mut();
                v.widgets.inactive.rounding  = Rounding::same(4.0);
                v.widgets.active.rounding    = Rounding::same(4.0);
                v.widgets.hovered.rounding   = Rounding::same(4.0);
                v.widgets.inactive.bg_fill   = rgb(20, 20, 20);
                v.widgets.active.bg_fill     = rgb(20, 20, 20);
                v.widgets.hovered.bg_fill    = rgb(25, 25, 25);
                v.widgets.inactive.bg_stroke = Stroke::new(1.0, rgb(60, 60, 60));
                v.widgets.active.bg_stroke   = Stroke::new(1.0, rgb(100, 100, 255));
                v.widgets.hovered.bg_stroke  = Stroke::new(1.0, rgb(80, 80, 80));
                ui.add(TextEdit::singleline(&mut username)
                    .hint_text("Username")
                    .desired_width(f32::INFINITY));
            });
            // Email copy 2 of 2
            ui.scope(|ui| {
                let v = ui.visuals_mut();
                v.widgets.inactive.rounding  = Rounding::same(4.0);
                v.widgets.active.rounding    = Rounding::same(4.0);
                v.widgets.hovered.rounding   = Rounding::same(4.0);
                v.widgets.inactive.bg_fill   = rgb(20, 20, 20);
                v.widgets.active.bg_fill     = rgb(20, 20, 20);
                v.widgets.hovered.bg_fill    = rgb(25, 25, 25);
                v.widgets.inactive.bg_stroke = Stroke::new(1.0, rgb(60, 60, 60));
                v.widgets.active.bg_stroke   = Stroke::new(1.0, rgb(100, 100, 255));
                v.widgets.hovered.bg_stroke  = Stroke::new(1.0, rgb(80, 80, 80));
                ui.add(TextEdit::singleline(&mut email)
                    .hint_text("Email")
                    .desired_width(f32::INFINITY));
            });
            // Buttons with custom hover - another scope+visuals_mut each
            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    let v = ui.visuals_mut();
                    v.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    v.widgets.hovered.bg_fill  = rgb(40, 40, 40);
                    ui.button("Cancel");
                });
                ui.scope(|ui| {
                    let v = ui.visuals_mut();
                    v.widgets.inactive.bg_fill = rgb(60, 60, 255);
                    v.widgets.hovered.bg_fill  = rgb(80, 80, 255);
                    v.widgets.active.bg_fill   = rgb(40, 40, 200);
                    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
                    ui.button("Save");
                });
            });
        });
});
```

</details>

<br/>

Same UI, ~50 lines vs ~30 and the raw-egui side is mostly visuals_mut copy-paste. Per-widget hover/focus colors. Once you start [composing styles](#composing-styles), the duplication on the After side collapses too.

<br/>

Another common pattern: a blinking prompt that must not reflow surrounding layout. With raw egui you reach for `allocate_exact_size` + `painter().text()`; with `egui_styled` it's two builder methods:

```rust
// Raw egui
let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::hover());
if visible {
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, font, color);
}

// egui_styled
Styled::label(text)
    .font(font)
    .text_color(color)
    .min_height(row_height)  // reserves the slot every frame
    .visible(visible)        // skips painting without collapsing space
    .show(ui);
```

<br/>

> **Fair comparison:** you can also factor the raw-egui side into helper functions, which closes most of the line gap (~25 vs ~20). The bigger remaining win is *what kind* of helper you can write: in raw egui, your helper has to take `&mut Ui` and render immediately. In `egui_styled` a style helper returns `impl Fn(W) -> W`, a pure value you can store, compose with other helpers, or tweak per call site (`.apply(primary_button(&t)).margin_top(8.0)`). Uniformity matters too: every helper in `egui_styled` has the same shape regardless of whether it styles a button, frame, or input.

## Features

| Status | Feature |
| :----: | :------ |
| ✅ | `StyledButton` with bg / hover_bg / active_bg / text_color / border / corner_radius |
| ✅ | `StyledLabel` with font_size, bold, italics, `wrap_mode` (wrap / truncate / extend) |
| ✅ | `StyledLabel` text effects: `.text_shadow()` / `.outline()` / `.glow()` / `.scale()` |
| ✅ | `StyledTextEdit` with hint, password, multiline, focus state styling |
| ✅ | `StyledCheckbox` with full pseudo-state support |
| 🚧 | `StyledSlider` generic over `T: Numeric`, but track/handle styling is shallow |
| 🚧 | `StyledComboBox` trigger styled, popup items inherit |
| ✅ | `StyledFrame` with bg / border / padding / margin / corner_radius |
| ✅ | `StyledRow` / `StyledColumn` containers with gap support |
| ✅ | `StyledStack` overlay container (layered children, offsets, alignment) |
| ✅ | `SharedStyle` resolver, hover/focus/active falls through to egui defaults |
| ✅ | `PseudoState` tracking via `egui::Memory` (1-frame lag, imperceptible) |
| ✅ | `StyledTheme` design tokens (colors / spacing / radii / typography / shadow / glow) |
| ✅ | `ThemeExt` for `egui::Context` (`ctx.set_styled_theme()` / `ctx.styled_theme()`) |
| ✅ | `Apply` trait for composable style functions |
| ✅ | Snapshot/visual regression tests via `egui_kittest` |
| ⚒️ | `Style` newtype as data (build once, merge into any widget) [future work] |

## Installation

```toml
[dependencies]
egui_styled = "0.4"
egui = "0.34"
```

## Design tokens

`egui_styled` ships **geometry/typography tokens** in [`StyledTheme`] (universally useful - spacing, radii, font sizes, font families) and an **optional starter color palette** in [`WebPalette`] (semantic web-style color names). Colors are inherently domain-specific, so the library doesn't force its vocabulary on you - for games, IDEs, or anything else, define your own struct and store it via the generic `DesignSlots` mechanism.

### Geometry (`StyledTheme`)

```rust
use egui::{CornerRadius, FontFamily, vec2};
use egui_styled::prelude::*;

pub fn geometry() -> StyledTheme {
    StyledTheme {
        rounding_sm: CornerRadius::same(2),
        rounding_md: CornerRadius::same(4),
        rounding_lg: CornerRadius::same(8),
        rounding_full: CornerRadius::same(u8::MAX),
        spacing_xs: 2.0, spacing_sm: 4.0, spacing_md: 8.0, spacing_lg: 16.0, spacing_xl: 32.0,
        font_size_sm: 12.0, font_size_md: 14.0, font_size_lg: 18.0, font_size_xl: 24.0,
        // Swap for `FontFamily::Name("YourFont".into())` after registering
        // the font with egui at startup.
        font_family_display: FontFamily::Proportional,
        font_family_body:    FontFamily::Proportional,
        font_family_mono:    FontFamily::Monospace,
        // Text effect scale tokens (used with .text_shadow() / .glow() on StyledLabel)
        shadow_sm: vec2(0.0, 1.0), shadow_md: vec2(0.0, 2.0), shadow_lg: vec2(0.0, 4.0),
        glow_sm: 4.0, glow_md: 8.0, glow_lg: 16.0,
    }
}
```

Pair the family tokens with the size scale at the call site via the helper methods:

```rust
Styled::label("Score")
    .font(theme.font_display(theme.font_size_xl))
    .show(ui);
```

### Colors - Option A: use the starter palette

If your app fits a web/dashboard vocabulary (`accent`, `error`, `warning`, `success`, `fg_on_accent`, etc.), use [`WebPalette`]:

```rust
use egui::Color32;
use egui_styled::prelude::*;

pub fn dark_palette() -> WebPalette {
    WebPalette {
        bg_primary:    Color32::from_rgb(15, 15, 15),
        bg_secondary:  Color32::from_rgb(20, 20, 20),
        bg_surface:    Color32::from_rgb(30, 30, 30),
        bg_elevated:   Color32::from_rgb(40, 40, 40),
        fg_primary:    Color32::from_gray(240),
        fg_secondary:  Color32::from_gray(180),
        fg_muted:      Color32::from_gray(120),
        fg_on_accent:  Color32::WHITE,
        accent:        Color32::from_rgb(60, 60, 255),
        accent_hover:  Color32::from_rgb(80, 80, 255),
        accent_active: Color32::from_rgb(40, 40, 200),
        error:         Color32::from_rgb(255, 80, 80),
        warning:       Color32::from_rgb(255, 180, 60),
        success:       Color32::from_rgb(80, 200, 120),
        border:        Color32::from_rgb(60, 60, 60),
        border_focus:  Color32::from_rgb(100, 100, 255),
    }
}
```

### Colors - Option B: define your own

If your app has domain-specific colors that don't fit web semantics (game HUDs, IDE syntax, etc.), define your own struct and store it the same way:

```rust
use egui::{Color32, FontFamily};
use egui_styled::prelude::*;

#[derive(Clone, Default)]
struct ArcadeColors {
    pub hud_glow:      Color32,
    pub enemy_red:     Color32,
    pub powerup_yellow: Color32,
    pub score_bg:      Color32,
}
```

### Storing and reading

Both `StyledTheme` and any user-defined type are stored on `egui::Context` via the same primitive:

```rust
// Once at startup
ctx.set_styled_theme(geometry());
ctx.set_design_data(dark_palette());     // or ArcadeColors { ... }

// Anywhere in your UI
let t = ui.ctx().styled_theme();
let p = ui.ctx().design_data::<WebPalette>();   // or ::<ArcadeColors>()
```

`DesignSlots` is the underlying typed-storage trait - one slot per `TypeId`. `ThemeExt::set_styled_theme` / `styled_theme` are convenience wrappers over it. If you need two slots of the same underlying type (two `Vec<Color32>` palettes, etc.), newtype them.

See [`examples/theme_demo.rs`](examples/theme_demo.rs) for two themes (midnight, parchment) you can use as starting points.

## Composing styles

Reuse styling across call sites with the `Apply` trait. Since colors and geometry are separate, helpers typically close over both:

```rust
fn primary_button(t: &StyledTheme, p: &WebPalette)
    -> impl Fn(StyledButton) -> StyledButton + 'static
{
    let (t, p) = (t.clone(), p.clone());
    move |b| {
        b.bg(p.accent)
         .hover_bg(p.accent_hover)
         .active_bg(p.accent_active)
         .text_color(p.fg_on_accent)
         .corner_radius(t.rounding_md)
    }
}

Styled::button("Save").apply(primary_button(&t, &p)).show(ui);
```

`Apply` is implemented for every styled type and is in the prelude.

The library doesn't ship preset helpers like `primary_button` - what's "primary" is a product decision, not a library one. Define them in your app the way above, alongside whatever color type you've chosen.

## Overlays (`Styled::stack()`)

`row` and `column` are thin wrappers over egui's flow `Layout`, so they can't put two things in the *same* pixels. `Styled::stack()` is the exception - an overlay container where every layer shares one origin and paints on top of the previous one (first layer = bottom, last = top). You'd otherwise have to drop down to raw `painter()` calls to achieve the z-index style effect: layered backgrounds, badges, "centered text over an image", or ghost/chromatic-aberration effects.

```rust
// Chromatic-aberration "[ENTER]": three offset layers.
Styled::stack()
    .layer_offset(vec2(-1.5, 0.0), |ui| {
        Styled::label("[ENTER]").text_color(cyan).extend().show(ui);
    })
    .layer_offset(vec2(1.5, 0.0), |ui| {
        Styled::label("[ENTER]").text_color(magenta).extend().show(ui);
    })
    .layer(|ui| {
        Styled::label("[ENTER]").text_color(white).extend().show(ui);
    })
    .show(ui);

// Overlay aligned within the first layer (e.g. centered over a background).
Styled::stack()
    .layer(|ui| background.show(ui))
    .layer_aligned(Align2::CENTER_CENTER, |ui| {
        Styled::label("GAME OVER").extend().show(ui);
    })
    .show(ui);
```

- `.layer(fn)` / `.layer_offset(offset, fn)` anchor at the shared origin (optionally nudged by a pixel offset).
- `.layer_aligned(align, fn)` positions a layer within the union of all *preceding* layers - so "background first, overlay aligned on it" works.
- `.layer_fixed(size, align, fn)` contributes only `size` to the stack's allocation; content is positioned within that box via `align` and may overflow freely. Use for scale-punch / bounce / pop animations where the content briefly renders larger than its resting size without pushing siblings.
- `.sense(Sense::click())` makes the whole stack report interaction; defaults to hover.

Because layers are painted before the stack's final position is known (it's the parent that decides, especially under centering), the stack paints at a provisional spot and then translates its own shapes into place. Two practical notes: `.extend()` on labels (see below) avoids truncation when a layer is laid out tight, and **interactive widgets inside a stack are hit-tested at the pre-translation position** - fine for the labels/visuals this is meant for, but don't bury a `button` deep inside a centered stack.

> **`.extend()`** - `StyledLabel` exposes egui's full `TextWrapMode` via `.wrap_mode(...)`, plus `.wrap()` / `.truncate()` / `.extend()` shortcuts. `.extend()` ("lay out at natural width, never wrap or truncate") is what keeps text intact inside tight rows and stacks.

## Text effects (`StyledLabel`)

`StyledLabel` exposes four glyph-level appearance primitives. They all stamp the laid-out galley at offsets and colors rather than painting offset rectangles — so effects follow the actual letterforms, not a bounding box.

All methods take plain per-frame values. **Animation stays consumer-side**: compute scale, intensity, or offset using `ctx.data` / `request_repaint` in your own code, then pass the current value to the builder.

```rust
// Drop shadow (token offset, or any Vec2)
Styled::label("SCORE")
    .text_shadow(theme.shadow_md, Color32::BLACK.linear_multiply(0.5))
    .text_color(Color32::WHITE)
    .show(ui);

// Chromatic aberration — two opposite-offset shadows, one label
Styled::label("[ENTER]")
    .text_shadow(vec2(-2.0, 0.0), cyan)
    .text_shadow(vec2( 2.0, 0.0), magenta)
    .text_color(Color32::WHITE)
    .extend()
    .show(ui);

// Faux stroke outline (8 compass-direction stamps)
Styled::label("GAME OVER")
    .outline(2.0, Color32::BLACK)
    .text_color(Color32::WHITE)
    .show(ui);

// Soft glow — intensity from 0.0 (invisible) to 1.0 (full)
Styled::label("000123456")
    .glow(Color32::from_rgb(0, 220, 255), theme.glow_md, intensity)
    .text_color(Color32::WHITE)
    .show(ui);

// Scale about a pivot — allocated footprint stays at natural size
// Pair with Styled::stack().layer_fixed() to prevent overflow affecting siblings
let resting = vec2(200.0, 40.0);
Styled::stack()
    .layer_fixed(resting, Align2::CENTER_CENTER, |ui| {
        Styled::label("000123456")
            .scale(factor, Align2::CENTER_CENTER)   // factor from your animation
            .text_color(Color32::WHITE)
            .show(ui);
    })
    .show(ui);
```

Effects compose — a single label can carry glow + chromatic shadows at once.

### Theme tokens

`StyledTheme` includes a shadow and glow scale that follows the same sm/md/lg ramp as spacing and rounding:

| Token | Default |
|-------|---------|
| `shadow_sm` | `vec2(0, 1)` |
| `shadow_md` | `vec2(0, 2)` |
| `shadow_lg` | `vec2(0, 4)` |
| `glow_sm` | 4 px |
| `glow_md` | 8 px |
| `glow_lg` | 16 px |

### Glow quality

egui has no blur pass, so glyph-shaped glow is faked by stamping the text many times at faint offsets distributed on a sunflower (Vogel) disk — an aperiodic golden-angle spiral with no grid or spokes, so the overlapping copies blend into a smooth halo that follows the letterforms. `samples` is a base density (count at an 8px radius); the real count scales with radius² so large glows stay smooth, and brightness is independent of the value. This is the priciest primitive; the default (64) blends cleanly at typical sizes:

```rust
// Cheaper — for many simultaneous glowing labels
.glow_quality(32)

// Smoother — for large hero text with a big radius
.glow_quality(96)
```

## Examples

```bash
cargo run --example basic              # buttons + frame
cargo run --example containers         # row / column / nesting
cargo run --example stack              # overlay container: offsets + alignment
cargo run --example text_effects       # shadow, outline, glow, scale — animated demos
cargo run --example text_edit          # focus state styling
cargo run --example theme_demo         # live theme switching with swatches
cargo run --example composable_styles  # Apply + reusable style functions
cargo run --example all_widgets        # every widget in one screen
cargo run --example game_over          # full game-over screen - Area + Column + Stack + custom palette
```

## Performance

Per styled widget the overhead vs raw egui is approximately:
- 1 `ui.scope` (which egui itself uses constantly internally)
- 2 `egui::Memory` lookups (pseudo-state load/store)
- 1 `SharedStyle::resolve` (a branch chain over `Option`s)

Mostly stack work, plus a few small heap allocations per widget: a `Visuals` clone, the scope's child `Ui` state, occasional short strings when a font override round-trips through `RichText` on text widgets.

## Status

Pre-1.0. The API surface is functional but not used in anger. Expect rough edges, especially around slider and combo box styling. Feedback and bug reports welcome.

## License

MIT or Apache-2.0, at your option.
