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

> **Fair comparison:** you can also factor the raw-egui side into helper functions, which closes most of the line gap (~25 vs ~20). The bigger remaining win is *what kind* of helper you can write: in raw egui, your helper has to take `&mut Ui` and render immediately. In `egui_styled` a style helper returns `impl Fn(W) -> W`, a pure value you can store, compose with other helpers, or tweak per call site (`.apply(primary_button(&t)).margin_top(8.0)`). Uniformity matters too: every helper in `egui_styled` has the same shape regardless of whether it styles a button, frame, or input.

## Features

| Status | Feature |
| :----: | :------ |
| ✅ | `StyledButton` with bg / hover_bg / active_bg / text_color / border / corner_radius |
| ✅ | `StyledLabel` with font_size, bold, italics, wrap |
| ✅ | `StyledTextEdit` with hint, password, multiline, focus state styling |
| ✅ | `StyledCheckbox` with full pseudo-state support |
| 🚧 | `StyledSlider` generic over `T: Numeric`, but track/handle styling is shallow |
| 🚧 | `StyledComboBox` trigger styled, popup items inherit |
| ✅ | `StyledFrame` with bg / border / padding / margin / corner_radius |
| ✅ | `StyledRow` / `StyledColumn` containers with gap support |
| ✅ | `SharedStyle` resolver, hover/focus/active falls through to egui defaults |
| ✅ | `PseudoState` tracking via `egui::Memory` (1-frame lag, imperceptible) |
| ✅ | `StyledTheme` design tokens (colors / spacing / radii / typography) |
| ✅ | `ThemeExt` for `egui::Context` (`ctx.set_styled_theme()` / `ctx.styled_theme()`) |
| ✅ | `Apply` trait for composable style functions |
| ⚒️ | `Style` newtype as data (build once, merge into any widget) [future work] |
| ⚒️ | Snapshot/visual regression tests [future work] |

## Installation

```toml
[dependencies]
egui_styled = "0.1"
egui = "0.34"
```

## Design tokens

`egui_styled` ships **geometry/typography tokens** in [`StyledTheme`] (universally useful — spacing, radii, font sizes, font families) and an **optional starter color palette** in [`WebPalette`] (semantic web-style color names). Colors are inherently domain-specific, so the library doesn't force its vocabulary on you — for games, IDEs, or anything else, define your own struct and store it via the generic `DesignSlots` mechanism.

### Geometry (`StyledTheme`)

```rust
use egui::{CornerRadius, FontFamily};
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
    }
}
```

Pair the family tokens with the size scale at the call site via the helper methods:

```rust
Styled::label("Score")
    .font(theme.font_display(theme.font_size_xl))
    .show(ui);
```

### Colors — Option A: use the starter palette

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

### Colors — Option B: define your own

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

`DesignSlots` is the underlying typed-storage trait — one slot per `TypeId`. `ThemeExt::set_styled_theme` / `styled_theme` are convenience wrappers over it. If you need two slots of the same underlying type (two `Vec<Color32>` palettes, etc.), newtype them.

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

The library doesn't ship preset helpers like `primary_button` — what's "primary" is a product decision, not a library one. Define them in your app the way above, alongside whatever color type you've chosen.

## Examples

```bash
cargo run --example basic              # buttons + frame
cargo run --example containers         # row / column / nesting
cargo run --example text_edit          # focus state styling
cargo run --example theme_demo         # live theme switching with swatches
cargo run --example composable_styles  # Apply + reusable style functions
cargo run --example all_widgets        # everything in one screen
```

## Performance

Per styled widget the overhead vs raw egui is approximately:
- 1 `ui.scope`
- 2 `egui::Memory` lookups (pseudo-state load/store) 
- 1 `SharedStyle::resolve` (a branch chain over `Option`s). 

Under 1μs per widget.

## Status

Pre-1.0. The API surface is functional but not used in anger. Expect rough edges, especially around slider and combo box styling. Feedback and bug reports welcome.

## License

MIT or Apache-2.0, at your option.
