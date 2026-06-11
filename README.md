# egui_styled

A styling layer for [egui](https://github.com/emilk/egui) with per-widget
hover/focus/active styling, semantic design tokens, and composable style
functions.

Tailwind-style utility styling for egui with Flutter-like
builder widget APIs. Style widgets in terms of 'how it looks' instead of
repeatedly cloning and mutating `ui.visuals_mut()`.

[docs.rs](https://docs.rs/egui_styled) ·
[crates.io](https://crates.io/crates/egui_styled) ·
[examples](examples/)

---

## Quick start

Install your geometry theme and palette once during app startup, then read them
anywhere in your UI.

```rust
use egui::Color32;
use egui_styled::prelude::*;

#[derive(Default)]
struct App {
    name: String,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        ctx.set_styled_theme(my_theme());
        ctx.set_design_data(my_palette());
        Self::default()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let (t, p) = ui.ctx().design::<WebPalette>();

        Styled::frame()
            .bg(p.bg_surface)
            .corner_radius(t.rounding_lg)
            .padding(t.spacing_lg)
            .border(1.0, p.border)
            .show(ui, |ui| {
                Styled::text_edit(&mut self.name)
                    .hint("Username")
                    .full_width()
                    .bg(p.bg_secondary)
                    .corner_radius(t.rounding_md)
                    .border(1.0, p.border)
                    .focus_border(1.0, p.border_focus)
                    .show(ui);

                Styled::button("Save")
                    .bg(p.accent)
                    .hover_bg(p.accent_hover)
                    .text_color(p.fg_on_accent)
                    .corner_radius(t.rounding_md)
                    .show(ui);
            });
    }
}

fn my_theme() -> StyledTheme {
    StyledTheme::default()
}

fn my_palette() -> WebPalette {
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

---

## Why this exists

egui styling lives on `ui.visuals_mut()`, which is global to the current `Ui`.
Applying a custom hover color to a button means cloning visuals, mutating
multiple `WidgetVisuals` states, and wrapping the widget in `ui.scope`. The
pattern works, but scales badly.

`egui_styled` wraps common egui widgets in builder APIs that handle that
boilerplate for you. Per-widget hover/focus/active states just work, and
styles become values you can compose instead of side-effecting helper
functions that must immediately render into a `Ui`.

---

## Before / After

A card with a text input and two buttons:

### With `egui_styled`

```rust
use egui::Color32;
use egui_styled::prelude::*;

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

                ui.add(
                    TextEdit::singleline(&mut username)
                        .hint_text("Username")
                        .desired_width(f32::INFINITY),
                );
            });

            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    let v = ui.visuals_mut();
                    v.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    v.widgets.hovered.bg_fill  = rgb(40, 40, 40);
                    ui.button("Cancel");
                });

                ui.scope(|ui| {
                    let v = ui.visuals_mut();
                    v.widgets.inactive.bg_fill   = rgb(60, 60, 255);
                    v.widgets.hovered.bg_fill    = rgb(80, 80, 255);
                    v.widgets.active.bg_fill     = rgb(40, 40, 200);
                    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
                    ui.button("Save");
                });
            });
        });
});
```

</details>

<br/>

> **Fair comparison:** you can factor the raw-egui version into helper
> functions, which closes most of the line gap. The bigger win is *what kind*
> of helper you can write. In raw egui, a style helper typically takes
> `&mut Ui` and renders immediately. In `egui_styled`, a style helper can
> return `impl Fn(W) -> W`: a pure value you can store, compose, and tweak at
> the call site (`.apply(primary_button(&t, &p)).corner_radius(0.0)`).

---

## Installation

```toml
[dependencies]
egui_styled = "0.6"
egui = "0.34"
```

### Version compatibility

| egui_styled | egui | Rust (MSRV) |
|-------------|------|-------------|
| 0.6         | 0.34 | 1.92        |
| 0.5.x       | 0.34 | 1.92        |

Tested with `eframe` on Linux, macOS, and Windows.

Web (wasm) is supported — a live demo runs at [david-spray.me](https://david-spray.me).

No feature flags currently.

---

## App setup

Install the theme and palette once during app startup, then read them anywhere
in your frame.

```rust
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "My App",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_styled_theme(my_geometry());
            cc.egui_ctx.set_design_data(my_colors());
            Ok(Box::new(App::default()))
        }),
    )
}

fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    let (t, p) = ctx.design::<MyPalette>();
    // … render with t and p
}
```

`design::<T>()` returns `(StyledTheme, T)`. If you only need the palette, use
`design_data::<T>()`.

---

## Core concepts

### Styled widgets

`egui_styled` provides styled wrappers for common egui widgets. Each one is a
builder that resolves its stored style into `WidgetVisuals` at `.show()` time,
inside its own `ui.scope`, so it does not pollute sibling widgets.

```rust
Styled::button("Save")
    .bg(accent)
    .hover_bg(accent_hover)
    .active_bg(accent_active)
    .text_color(Color32::WHITE)
    .corner_radius(4.0)
    .show(ui);

Styled::text_edit(&mut value)
    .hint("Type here…")
    .full_width()
    .bg(input_bg)
    .border(1.0, border)
    .focus_border(1.0, border_focus)
    .show(ui);

Styled::label("Status")
    .font_size(12.0)
    .text_color(muted)
    .bold()
    .show(ui);
```

Available today: `StyledButton`, `StyledLabel`, `StyledTextEdit`,
`StyledCheckbox`, `StyledSlider` (partial), `StyledComboBox` (partial).

### Design tokens

`egui_styled` separates geometry/typography tokens from colors.

**`StyledTheme`** holds reusable geometry and type-scale values — spacing,
radii, font sizes, font families, and text-effect scales:

```rust
StyledTheme {
    rounding_sm: CornerRadius::same(2),
    rounding_md: CornerRadius::same(4),
    rounding_lg: CornerRadius::same(8),
    spacing_xs: 2.0,  spacing_sm: 4.0,
    spacing_md: 8.0,  spacing_lg: 16.0,
    font_size_sm: 12.0, font_size_md: 14.0, font_size_lg: 18.0,
    font_family_body: FontFamily::Proportional,
    font_family_mono: FontFamily::Monospace,
    shadow_md: vec2(0.0, 2.0),
    glow_md: 8.0,
    // …
}
```

**`WebPalette`** is an optional starter palette for web/dashboard-style UIs:

```rust
WebPalette {
    bg_surface:   Color32::from_rgb(30, 30, 30),
    fg_primary:   Color32::from_gray(240),
    accent:       Color32::from_rgb(60, 60, 255),
    accent_hover: Color32::from_rgb(80, 80, 255),
    border:       Color32::from_rgb(60, 60, 60),
    border_focus: Color32::from_rgb(100, 100, 255),
    // …
}
```

If your app has its own design language, define your own palette type instead:

```rust
#[derive(Clone, Default)]
struct ArcadeColors {
    hud_glow:       Color32,
    enemy_red:      Color32,
    powerup_yellow: Color32,
}

ctx.set_design_data(ArcadeColors { hud_glow: cyan, … });
let colors = ctx.design_data::<ArcadeColors>();
```

Theme and palette data live on `egui::Context` in typed slots. If you need two
slots of the same underlying type, use newtypes.

### Composable styles

The `Apply` trait lets you define a style vocabulary in app code without
coupling helpers to a specific `Ui`:

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

// Tweak at the call site without a new helper:
Styled::button("Delete").apply(primary_button(&t, &p)).bg(p.error).show(ui);
```

The crate intentionally does not ship helpers like `primary_button` — what
counts as "primary" is a product decision, so you define that in
app code.

### Containers and layout

**`StyledRow`** / **`StyledColumn`** are flow containers with gap, alignment,
and all box styling (bg, border, padding, corner radius, shadow):

```rust
Styled::row().gap(8.0).show(ui, |ui| {
    Styled::button("A").show(ui);
    Styled::button("B").show(ui);
});
```

**`StyledFrame`** is the core styled box container. It supports padding,
border, corner radius, width/height constraints, vertical justify, and
background images.

**`StyledArea`** is a top-level positioned container for modals, toasts, and
backdrops. It works on `&egui::Context` rather than `&mut Ui`.

**`StyledStack`** is the overlay container: multiple layers share one origin
and paint on top of each other. Use it for layered backgrounds, badges,
centered text over images, or visual effects such as chromatic aberration:

```rust
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
```

> **Note on interactivity in stacks:** `StyledStack` translates its shapes
> after layout, so interactive widgets inside translated layers are hit-tested
> at the pre-translation position. Fine for labels and visual effects; avoid
> burying a button deep inside a centered stack.

> **`.extend()`** on `StyledLabel` uses `TextWrapMode::Extend`: natural width,
> no wrapping, no truncation. Especially useful inside stacks and tight rows.

### Layout primitives

`egui_styled` adds CSS-inspired layout utilities on top of egui's flow system.

**Spacer** — pushes following siblings to the far edge:

```rust
Styled::row().full_width().show(ui, |ui| {
    Styled::label("Left").show(ui);
    Styled::spacer().show(ui);
    Styled::label("Right").show(ui);  // pushed to the right edge
});
```

**Percentage sizing** — `width_pct` / `height_pct` on any styled type.
Resolves at render time, supersedes `full_width`/`full_height`, composes with
`min_*`/`max_*` as clamps:

```rust
Styled::frame().width_pct(50.0).show(ui, |ui| { /* half the available width */ });
Styled::frame().width_pct(50.0).max_width(200.0).show(ui, |ui| { /* capped */ });
```

**Aspect ratio** — derives height from width (`width ÷ height`, CSS
convention). Requires a definite width (`width_pct` or `full_width`); no-op
without one. Overridden by an explicit `height_pct` or `full_height`:

```rust
// 16:9 placeholder that scales with its column
Styled::frame()
    .width_pct(60.0)
    .aspect_ratio(16.0 / 9.0)
    .bg(placeholder_bg)
    .show(ui, |_| {});
```

**Distribution** — three CSS style `justify-content` modes on `StyledRow`. Each
measures item widths with an invisible first pass, then distributes slack:

```rust
// ends pinned, equal gaps between
Styled::row().full_width().bg(nav_bg).padding(10.0).space_between()
    .item(|ui| { Styled::button("Home").show(ui); })
    .item(|ui| { Styled::button("About").show(ui); })
    .item(|ui| { Styled::button("Contact").show(ui); })
    .show(ui);

// space_around: equal margin each side of each item
// space_evenly: equal space everywhere (before, between, after)
```

**Wrapping rows** — children flow onto new lines as the container narrows.
Uses a cross-frame measurement pass so styled widgets (which egui's native
`with_main_wrap` can't wrap) participate correctly:

```rust
let mut tags = Styled::row().full_width().gap(6.0).wrap();
for tag in ["egui", "rust", "ui", "responsive", "widgets"] {
    tags = tags.item(move |ui| {
        Styled::button(tag).corner_radius(12.0).show(ui);
    });
}
tags.show(ui);
```

### Text effects

`StyledLabel` exposes glyph-level effects by stamping the laid-out text at
offsets and colors, so the effect follows the letterforms rather than a
bounding box.

**Animation stays consumer-side**: compute intensity, scale, or offset in your
own state and pass the current value each frame.

```rust
// Drop shadow
Styled::label("SCORE")
    .text_shadow(theme.shadow_md, Color32::BLACK.linear_multiply(0.5))
    .text_color(Color32::WHITE)
    .show(ui);

// Chromatic aberration — two opposite shadows, one label
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

// Soft glow — intensity from 0.0 to 1.0
Styled::label("000123456")
    .glow(Color32::from_rgb(0, 220, 255), theme.glow_md, intensity)
    .text_color(Color32::WHITE)
    .show(ui);
```

Effects compose: a single label can carry glow, outline, multiple shadows, and
scale simultaneously.

Glow is the most expensive text primitive — egui has no blur pass, so it's
approximated by stamping the text many times on a Vogel disk. The
default quality suits typical UI usage; tune with `.glow_quality(n)` if needed.

### Images

`egui_styled` never loads or uploads textures itself. The app owns that part:
use `ctx.load_texture`, install URI loaders with
`egui_extras::install_image_loaders`, or use `egui::include_image!` for
compile-time bytes.

```rust
// Inline widget — icons, portraits, thumbnails
Styled::image(egui::include_image!("assets/icon.png"))
    .size(Vec2::splat(64.0))
    .corner_radius(8.0)
    .border(1.0, theme.border_color)
    .hover_tint(Color32::from_rgba_unmultiplied(255, 255, 255, 180))
    .shadow_filled(vec2(2.0, 2.0), Color32::from_black_alpha(100))
    .show(ui);

// Background texture behind children — same rounded-corner clipping as `bg`
Styled::area()
    .fill_screen()
    .background_image(egui::include_image!("assets/bg.jpg"))
    .background_image_fit(BackgroundImageFit::Cover)
    .show(ctx, |ui| { /* content in front */ });
```

Rounded-corner clipping uses the same textured shape path as egui's own
`Frame`, so background images and inline images respect `corner_radius`
correctly.

---

## Widget support

| Widget | Status |
|--------|--------|
| `StyledButton` | ✅ Full pseudo-state, shadow, padding, border |
| `StyledLabel` | ✅ Font, color, effects, min-height, visibility |
| `StyledTextEdit` | ✅ Hint, multiline, password, focus styling, padding, border |
| `StyledCheckbox` | ✅ Full pseudo-state |
| `StyledSlider` | 🚧 Partial — track/handle styling is shallow |
| `StyledComboBox` | 🚧 Partial — trigger styled, popup items inherit |
| `StyledFrame` | ✅ Full box model, background image, vertical justify, aspect ratio |
| `StyledRow` / `StyledColumn` | ✅ Gap, alignment, box styling, distribution, wrapping |
| `StyledStack` | ✅ Overlay layers, offsets, alignment, fixed-size layers |
| `StyledArea` | ✅ Top-level positioned container, fill_screen, anchor |
| `StyledImage` | ✅ Inline images, tint, hover tint, corner radius, border, shadow |
| `StyledSpacer` | ✅ Flex spacer |
| `background_image` | ✅ On box containers: stretch or cover fit, fade-in, tint |

---

## Limitations

Things that are intentionally out of scope or not yet solved:

- **Immediate-mode constraints still apply.** Features that need multi-pass
  measurement — distribution, wrapping, vertical justify — use an invisible
  measurement frame and settle on the next repaint.
- **Pseudo-state tracking has a 1-frame lag.** State is written to
  `egui::Memory` at the end of each frame and read at the start of the next.
  In practice this matches egui's frame-based interaction model and is not
  noticeable.
- **Interactive widgets inside translated stacks are not reliable.**
  `StyledStack` paints visually in the right place, but hit-testing still uses
  the untranslated rect.
- **`StyledSlider` and `StyledComboBox` are not deeply styleable yet.**
  Trigger-level styling works; deeper internals are still limited.
- **Not a CSS or flexbox engine.** Percentage sizing, wrapping, and
  distribution are implemented as egui-friendly utilities, not a constraint
  solver.
- **Not a retained widget system.**
- **Not an accessibility framework.** It can style focus states, but it does
  not add semantic roles or screen-reader support.

---

## Performance

`egui_styled` adds a small per-widget overhead on top of raw egui:

- 1 `ui.scope` -
- 2 `egui::Memory` lookups for pseudo-state load/store
- 1 `SharedStyle::resolve`
- A `Visuals` clone, a child `Ui` allocation, occasional short-lived strings
  for font override paths

For ordinary tool UIs this is usually fine. Profile if you render very large
numbers of styled widgets per frame.

**Text glow is the most expensive primitive** — it stamps the glyph run many
times. Lower `.glow_quality(n)` if you render lots of glowing labels
simultaneously.

Distribution and wrapping rows add a measurement frame on first render, then
settle. After that their steady-state overhead is similar to other styled
containers.

---

## Testing

Visual regression tests use [`egui_kittest`](https://github.com/emilk/egui/tree/master/crates/egui_kittest).
Baselines live in `tests/snapshots/`.

```bash
cargo test --lib   # unit tests (fast, no GPU)
cargo test         # all tests including kittest snapshots (requires wgpu)
```

---

## Examples

```bash
cargo run --example basic              # buttons + frame
cargo run --example containers         # row / column / nesting
cargo run --example layout             # spacer, pct sizing, wrapping, distribution, aspect ratio
cargo run --example stack              # overlay container: offsets + alignment
cargo run --example images             # StyledImage + background_image: rounding, tint, hover, cover
cargo run --example text_effects       # shadow, outline, glow, scale
cargo run --example text_edit          # focus state styling
cargo run --example theme_demo         # live theme switching
cargo run --example composable_styles  # Apply + reusable style functions
cargo run --example all_widgets        # every widget in one screen
cargo run --example game_over          # Area + Column + Stack + custom palette
```

---

## Status

Pre-1.0. The API is functional and covered by examples and tests, but not yet
tested in large production apps. Expect breaking changes between minor
versions until 1.0.

Feedback and bug reports are welcome — [open an issue](https://github.com/dspray95/egui_styled/issues).

---

## License

MIT or Apache-2.0, at your option.
