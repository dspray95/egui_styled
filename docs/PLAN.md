egui_styled — Full Architecture Plan
1. Module Structure
text

egui_styled/
├── src/
│   ├── lib.rs              # Public API re-exports, Styled namespace
│   ├── theme.rs            # Theme struct, design tokens, ThemeExt trait
│   ├── state.rs            # Pseudo-state tracking via egui::Memory
│   ├── style.rs            # SharedStyle, StyleOverride, color helpers
│   ├── apply.rs            # Apply trait for composable style functions
│   │
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── button.rs       # StyledButton
│   │   ├── label.rs        # StyledLabel
│   │   ├── text_edit.rs    # StyledTextEdit
│   │   ├── checkbox.rs     # StyledCheckbox
│   │   ├── slider.rs       # StyledSlider
│   │   ├── image.rs        # StyledImage (stretch goal)
│   │   └── combo_box.rs    # StyledComboBox (closure-based, special)
│   │
│   ├── containers/
│   │   ├── mod.rs
│   │   ├── frame.rs        # StyledFrame
│   │   ├── row.rs          # StyledRow
│   │   ├── column.rs       # StyledColumn
│   │   └── scroll.rs       # StyledScroll (stretch goal)
│   │
│   └── helpers.rs          # rgb(), rgba(), spacing helpers
│
├── examples/
│   ├── basic.rs
│   ├── theme_demo.rs
│   └── composable_styles.rs
│
├── Cargo.toml
└── README.md

Rationale: Widgets and containers are separated because they have fundamentally different show() signatures — widgets return Response, containers take a closure and return InnerResponse. Keeping them in separate modules makes this boundary explicit.
2. Core Types and Traits
2.1 SharedStyle — The unified style bag

Every styled widget carries this. It holds all the style properties that are common across widget types. Widget-specific properties live on the widget struct itself.
rust

// src/style.rs

/// Properties common to most styled widgets.
/// All fields are Option — None means "inherit from egui defaults."
#[derive(Clone, Default, Debug)]
pub struct SharedStyle {
    // Background
    pub bg: Option<Color32>,
    pub hover_bg: Option<Color32>,
    pub active_bg: Option<Color32>,
    pub focus_bg: Option<Color32>,

    // Text
    pub text_color: Option<Color32>,
    pub hover_text_color: Option<Color32>,
    pub font_size: Option<f32>,
    pub font_id: Option<FontId>,

    // Border
    pub border: Option<Stroke>,
    pub hover_border: Option<Stroke>,
    pub focus_border: Option<Stroke>,

    // Geometry
    pub rounding: Option<Rounding>,
    pub padding: Option<Margin>,  // egui calls it Margin but uses it for padding
    pub margin: Option<Margin>,

    // Sizing
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub full_width: bool,

    // Cursor
    pub cursor_icon: Option<CursorIcon>,
}

Key design decision: All Option<T>. This means we never override a style the user didn't explicitly set. At resolve time, None falls through to the current egui Visuals — no surprises.
2.2 ResolvedStyle — Flattened for the current frame
rust

// src/style.rs

/// The concrete style values for this frame, after resolving
/// pseudo-state and falling back to egui defaults.
pub struct ResolvedStyle {
    pub bg: Color32,
    pub text_color: Color32,
    pub border: Stroke,
    pub rounding: Rounding,
    pub padding: Margin,
    pub margin: Margin,
    pub cursor_icon: Option<CursorIcon>,
}

impl SharedStyle {
    /// Resolve against current pseudo-state and egui's active visuals.
    pub fn resolve(
        &self,
        state: PseudoState,
        default: &WidgetVisuals,
    ) -> ResolvedStyle {
        let bg = match state {
            _ if state.active && self.active_bg.is_some()
                => self.active_bg.unwrap(),
            _ if state.hovered && self.hover_bg.is_some()
                => self.hover_bg.unwrap(),
            _ if state.focused && self.focus_bg.is_some()
                => self.focus_bg.unwrap(),
            _ => self.bg.unwrap_or(default.bg_fill),
        };

        let border = match state {
            _ if state.focused && self.focus_border.is_some()
                => self.focus_border.unwrap(),
            _ if state.hovered && self.hover_border.is_some()
                => self.hover_border.unwrap(),
            _ => self.border.unwrap_or(default.bg_stroke),
        };

        // ... similarly for text_color, etc.

        ResolvedStyle { bg, text_color, border, rounding, padding, margin, cursor_icon }
    }
}

2.3 PseudoState — Last frame's interaction state
rust

// src/state.rs

#[derive(Clone, Copy, Default, Debug)]
pub struct PseudoState {
    pub hovered: bool,
    pub active: bool,   // mouse down on widget
    pub focused: bool,  // keyboard focus (text edits, etc.)
}

/// Unique key for a widget's pseudo-state in egui::Memory.
/// We use egui::Id, which the widget already has.
impl PseudoState {
    /// Read last frame's state from memory.
    pub fn load(ui: &Ui, id: Id) -> Self {
        ui.memory(|mem| {
            mem.data.get_temp::<PseudoState>(id).unwrap_or_default()
        })
    }

    /// Write this frame's state into memory for next frame.
    pub fn store(self, ui: &Ui, id: Id) {
        ui.memory_mut(|mem| {
            mem.data.insert_temp(id, self);
        });
    }

    /// Build from an egui::Response after rendering.
    pub fn from_response(response: &Response) -> Self {
        Self {
            hovered: response.hovered(),
            active: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
        }
    }
}

This is the core trick. The render cycle for any styled widget is:
text

1. Generate stable Id for this widget
2. Load PseudoState from last frame via egui::Memory
3. Resolve SharedStyle using that PseudoState
4. Apply resolved style (scoped visuals, Frame wrapper, etc.)
5. Render the actual egui widget, get Response
6. Build new PseudoState from Response, store it for next frame
7. Return Response

One frame of latency on state transitions. Imperceptible.
2.4 The Apply trait — Composable style functions
rust

// src/apply.rs

pub trait Apply: Sized {
    /// Apply a style function to this builder.
    fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

// Every styled widget implements Apply (blanket or manual).

Usage:
rust

fn card_style(f: StyledFrame) -> StyledFrame {
    f.bg(rgb(30, 30, 30))
     .rounding(8.0)
     .padding(16.0)
     .border(1.0, rgb(60, 60, 60))
}

fn primary_button(b: StyledButton) -> StyledButton {
    b.bg(rgb(60, 60, 255))
     .hover_bg(rgb(80, 80, 255))
     .active_bg(rgb(40, 40, 200))
     .text_color(Color32::WHITE)
     .rounding(4.0)
}

// Usage:
Styled::frame().apply(card_style).show(ui, |ui| {
    Styled::button("Save").apply(primary_button).show(ui);
});

3. Widget-to-Wrapper Mapping

Each widget type has its own struct that holds SharedStyle plus widget-specific data:
egui widget	egui_styled type	Widget-specific fields	show() signature
Button	StyledButton	text: WidgetText, image: Option<Image>, shortcut_text: Option<RichText>	fn show(self, ui: &mut Ui) -> Response
Label	StyledLabel	text: WidgetText, wrap: Option<bool>, truncate: bool	fn show(self, ui: &mut Ui) -> Response
TextEdit	StyledTextEdit	text: &mut String, hint: Option<String>, multiline: bool, password: bool	fn show(self, ui: &mut Ui) -> Response
Checkbox	StyledCheckbox	checked: &mut bool, label: WidgetText	fn show(self, ui: &mut Ui) -> Response
Slider	StyledSlider	value: &mut f64, range: RangeInclusive<f64>, text: Option<String>	fn show(self, ui: &mut Ui) -> Response
Frame	StyledFrame	(none beyond SharedStyle)	fn show(self, ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> InnerResponse<()>
Row	StyledRow	gap: Option<f32>	fn show(self, ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> InnerResponse<()>
Column	StyledColumn	gap: Option<f32>	fn show(self, ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> InnerResponse<()>
ComboBox	StyledComboBox	id: Id, selected_text: WidgetText, width: Option<f32>	fn show(self, ui: &mut Ui, f: impl FnOnce(&mut Ui)) -> InnerResponse<Option<()>>
The Styled Namespace
rust

// src/lib.rs

pub struct Styled;

impl Styled {
    pub fn button(text: impl Into<WidgetText>) -> StyledButton {
        StyledButton::new(text)
    }

    pub fn label(text: impl Into<WidgetText>) -> StyledLabel {
        StyledLabel::new(text)
    }

    pub fn text_edit(text: &mut String) -> StyledTextEdit<'_> {
        StyledTextEdit::new(text)
    }

    pub fn checkbox(checked: &mut bool, label: impl Into<WidgetText>) -> StyledCheckbox<'_> {
        StyledCheckbox::new(checked, label)
    }

    pub fn slider<'a>(value: &'a mut f64, range: RangeInclusive<f64>) -> StyledSlider<'a> {
        StyledSlider::new(value, range)
    }

    pub fn frame() -> StyledFrame {
        StyledFrame::new()
    }

    pub fn row() -> StyledRow {
        StyledRow::new()
    }

    pub fn column() -> StyledColumn {
        StyledColumn::new()
    }
}

4. Anatomy of a Styled Widget (Detailed Example)

Let's trace StyledButton end-to-end — every other widget follows the same pattern.
rust

// src/widgets/button.rs

pub struct StyledButton {
    text: WidgetText,
    image: Option<Image<'static>>,
    style: SharedStyle,
}

impl StyledButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            image: None,
            style: SharedStyle::default(),
        }
    }

    // ---- Widget-specific builders ----

    pub fn image(mut self, img: Image<'static>) -> Self {
        self.image = Some(img);
        self
    }

    // ---- Style builders (generated via macro or written out) ----

    pub fn bg(mut self, c: Color32) -> Self {
        self.style.bg = Some(c);
        self
    }

    pub fn hover_bg(mut self, c: Color32) -> Self {
        self.style.hover_bg = Some(c);
        self
    }

    pub fn active_bg(mut self, c: Color32) -> Self {
        self.style.active_bg = Some(c);
        self
    }

    pub fn text_color(mut self, c: Color32) -> Self {
        self.style.text_color = Some(c);
        self
    }

    pub fn rounding(mut self, r: impl Into<Rounding>) -> Self {
        self.style.rounding = Some(r.into());
        self
    }

    pub fn border(mut self, width: f32, color: Color32) -> Self {
        self.style.border = Some(Stroke::new(width, color));
        self
    }

    pub fn full_width(mut self) -> Self {
        self.style.full_width = true;
        self
    }

    pub fn margin_top(mut self, v: f32) -> Self {
        let m = self.style.margin.get_or_insert(Margin::ZERO);
        m.top = v;
        self
    }

    // ... etc for all SharedStyle fields

    // ---- The render method ----

    pub fn show(self, ui: &mut Ui) -> Response {
        // 1. Stable ID
        let id = ui.make_persistent_id(ui.next_auto_id());

        // 2. Load last frame's pseudo-state
        let pseudo = PseudoState::load(ui, id);

        // 3. Resolve style
        let visuals = ui.visuals().clone();
        let widget_vis = if pseudo.active {
            &visuals.widgets.active
        } else if pseudo.hovered {
            &visuals.widgets.hovered
        } else {
            &visuals.widgets.inactive
        };
        let resolved = self.style.resolve(pseudo, widget_vis);

        // 4. Apply margin (allocate space before the widget)
        if let Some(margin) = self.style.margin {
            ui.add_space(margin.top);
        }

        // 5. Apply sizing
        if self.style.full_width {
            ui.set_min_width(ui.available_width());
        }

        // 6. Scoped visual override — this is the key integration point
        let response = ui.scope(|ui| {
            // Override the widget visuals for all states within this scope
            let vis = ui.visuals_mut();
            vis.widgets.inactive.bg_fill = resolved.bg;
            vis.widgets.inactive.bg_stroke = resolved.border;
            vis.widgets.inactive.rounding = resolved.rounding;
            vis.widgets.hovered.bg_fill = resolved.bg; // already resolved for state
            vis.widgets.hovered.bg_stroke = resolved.border;
            vis.widgets.hovered.rounding = resolved.rounding;
            vis.widgets.active.bg_fill = resolved.bg;
            vis.widgets.active.bg_stroke = resolved.border;
            vis.widgets.active.rounding = resolved.rounding;

            if let Some(color) = self.style.text_color {
                vis.widgets.inactive.fg_stroke = Stroke::new(1.0, color);
                vis.widgets.hovered.fg_stroke = Stroke::new(1.0, color);
                vis.widgets.active.fg_stroke = Stroke::new(1.0, color);
            }

            // Build the actual egui widget
            let mut btn = egui::Button::new(self.text);
            if let Some(img) = self.image {
                btn = btn.image(img);
            }
            if self.style.full_width {
                btn = btn.min_size(egui::vec2(ui.available_width(), 0.0));
            }

            ui.add(btn)
        }).inner;

        // 7. Store this frame's state for next frame
        PseudoState::from_response(&response).store(ui, id);

        // 8. Handle cursor override
        if let Some(icon) = resolved.cursor_icon {
            if response.hovered() {
                ui.ctx().set_cursor_icon(icon);
            }
        }

        response
    }
}

impl Apply for StyledButton {}

Why ui.scope()? It creates a child Ui with cloned Style/Visuals. Our mutations are automatically scoped — when the scope ends, the parent Ui is untouched. This is how we coexist with raw egui code.
5. The Theme System
rust

// src/theme.rs

#[derive(Clone, Debug)]
pub struct Theme {
    // Semantic colors
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,

    pub fg_primary: Color32,
    pub fg_secondary: Color32,
    pub fg_muted: Color32,

    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,

    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,

    pub border: Color32,
    pub border_focus: Color32,

    // Geometry tokens
    pub rounding_sm: Rounding,
    pub rounding_md: Rounding,
    pub rounding_lg: Rounding,
    pub rounding_full: Rounding,

    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // Typography
    pub font_size_sm: f32,
    pub font_size_md: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg_primary: rgb(15, 15, 15),
            bg_secondary: rgb(20, 20, 20),
            bg_surface: rgb(30, 30, 30),
            bg_elevated: rgb(40, 40, 40),

            fg_primary: Color32::from_gray(240),
            fg_secondary: Color32::from_gray(180),
            fg_muted: Color32::from_gray(120),

            accent: rgb(60, 60, 255),
            accent_hover: rgb(80, 80, 255),
            accent_active: rgb(40, 40, 200),

            error: rgb(255, 80, 80),
            warning: rgb(255, 180, 60),
            success: rgb(80, 200, 120),

            border: rgb(60, 60, 60),
            border_focus: rgb(100, 100, 255),

            rounding_sm: Rounding::same(2.0),
            rounding_md: Rounding::same(4.0),
            rounding_lg: Rounding::same(8.0),
            rounding_full: Rounding::same(999.0),

            spacing_xs: 2.0,
            spacing_sm: 4.0,
            spacing_md: 8.0,
            spacing_lg: 16.0,
            spacing_xl: 32.0,

            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 18.0,
            font_size_xl: 24.0,
        }
    }

    pub fn light() -> Self {
        // ... inverse palette
        todo!()
    }
}

Theme Storage and Access

The theme lives in egui::Memory via a type-map, so it's accessible from anywhere without passing it through function arguments:
rust

// src/theme.rs

/// Extension trait on egui::Context for theme access.
pub trait ThemeExt {
    fn set_theme(&self, theme: Theme);
    fn theme(&self) -> Theme;
}

impl ThemeExt for egui::Context {
    fn set_theme(&self, theme: Theme) {
        self.memory_mut(|mem| {
            mem.data.insert_persisted(egui::Id::NULL, theme);
        });
    }

    fn theme(&self) -> Theme {
        self.memory(|mem| {
            mem.data
                .get_persisted::<Theme>(egui::Id::NULL)
                .unwrap_or_default()
        })
    }
}

/// Convenience for use inside widget implementations.
pub(crate) fn current_theme(ui: &Ui) -> Theme {
    ui.ctx().theme()
}

Then users can write theme-aware style functions:
rust

fn primary_button(b: StyledButton, theme: &Theme) -> StyledButton {
    b.bg(theme.accent)
     .hover_bg(theme.accent_hover)
     .active_bg(theme.accent_active)
     .text_color(Color32::WHITE)
     .rounding(theme.rounding_md)
}

// Or, using the context-based approach:
fn primary_button_auto(ui: &Ui, b: StyledButton) -> StyledButton {
    let t = ui.ctx().theme();
    b.bg(t.accent).hover_bg(t.accent_hover).active_bg(t.accent_active)
     .text_color(Color32::WHITE).rounding(t.rounding_md)
}

6. Containers: Frame, Row, Column
StyledFrame
rust

// src/containers/frame.rs

pub struct StyledFrame {
    style: SharedStyle,
}

impl StyledFrame {
    pub fn new() -> Self {
        Self { style: SharedStyle::default() }
    }

    // ... all the builder methods (bg, rounding, padding, border, margin, etc.)

    pub fn show(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<()> {
        // Apply margin
        if let Some(m) = self.style.margin {
            ui.add_space(m.top);
        }

        // Build an egui::Frame from our resolved style
        let mut frame = egui::Frame::default();
        if let Some(bg) = self.style.bg {
            frame = frame.fill(bg);
        }
        if let Some(r) = self.style.rounding {
            frame = frame.rounding(r);
        }
        if let Some(p) = self.style.padding {
            frame = frame.inner_margin(p);
        }
        if let Some(b) = self.style.border {
            frame = frame.stroke(b);
        }

        frame.show(ui, |ui| {
            if self.style.full_width {
                ui.set_min_width(ui.available_width());
            }
            add_contents(ui);
        })
    }
}

StyledRow / StyledColumn
rust

// src/containers/row.rs

pub struct StyledRow {
    gap: Option<f32>,
    style: SharedStyle,
}

impl StyledRow {
    pub fn new() -> Self {
        Self { gap: None, style: SharedStyle::default() }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<()> {
        // Wrap in a StyledFrame if any frame-level styles are set,
        // then use horizontal_with_main_wrap or horizontal
        let inner_fn = |ui: &mut Ui| {
            if let Some(gap) = self.gap {
                ui.spacing_mut().item_spacing.x = gap;
            }
            ui.horizontal(|ui| {
                add_contents(ui);
            })
            .inner
        };

        if self.style.has_frame_styles() {
            // Delegate to StyledFrame logic
            StyledFrame { style: self.style }
                .show(ui, |ui| { inner_fn(ui); })
        } else {
            let ir = ui.horizontal(|ui| {
                if let Some(gap) = self.gap {
                    ui.spacing_mut().item_spacing.x = gap;
                }
                add_contents(ui);
            });
            InnerResponse::new((), ir.response)
        }
    }
}

StyledColumn is identical but uses ui.vertical() and item_spacing.y.
7. Reducing Builder Boilerplate: The Macro

Every styled type needs the same ~20 builder methods for SharedStyle fields. A macro handles this:
rust

// src/style.rs

macro_rules! impl_shared_style_methods {
    ($ty:ty) => {
        impl $ty {
            pub fn bg(mut self, c: Color32) -> Self {
                self.style.bg = Some(c); self
            }
            pub fn hover_bg(mut self, c: Color32) -> Self {
                self.style.hover_bg = Some(c); self
            }
            pub fn active_bg(mut self, c: Color32) -> Self {
                self.style.active_bg = Some(c); self
            }
            pub fn focus_bg(mut self, c: Color32) -> Self {
                self.style.focus_bg = Some(c); self
            }
            pub fn text_color(mut self, c: Color32) -> Self {
                self.style.text_color = Some(c); self
            }
            pub fn hover_text_color(mut self, c: Color32) -> Self {
                self.style.hover_text_color = Some(c); self
            }
            pub fn rounding(mut self, r: impl Into<Rounding>) -> Self {
                self.style.rounding = Some(r.into()); self
            }
            pub fn border(mut self, width: f32, color: Color32) -> Self {
                self.style.border = Some(Stroke::new(width, color)); self
            }
            pub fn hover_border(mut self, width: f32, color: Color32) -> Self {
                self.style.hover_border = Some(Stroke::new(width, color)); self
            }
            pub fn focus_border(mut self, width: f32, color: Color32) -> Self {
                self.style.focus_border = Some(Stroke::new(width, color)); self
            }
            pub fn padding(mut self, p: impl Into<Margin>) -> Self {
                self.style.padding = Some(p.into()); self
            }
            pub fn margin_top(mut self, v: f32) -> Self {
                let m = self.style.margin.get_or_insert(Margin::ZERO);
                m.top = v; self
            }
            pub fn margin_bottom(mut self, v: f32) -> Self {
                let m = self.style.margin.get_or_insert(Margin::ZERO);
                m.bottom = v; self
            }
            pub fn full_width(mut self) -> Self {
                self.style.full_width = true; self
            }
            pub fn min_width(mut self, w: f32) -> Self {
                self.style.min_width = Some(w); self
            }
            pub fn max_width(mut self, w: f32) -> Self {
                self.style.max_width = Some(w); self
            }
            pub fn font_size(mut self, s: f32) -> Self {
                self.style.font_size = Some(s); self
            }
        }
    };
}

// Usage in each widget file:
impl_shared_style_methods!(StyledButton);
impl_shared_style_methods!(StyledTextEdit<'_>);
impl_shared_style_methods!(StyledFrame);
// etc.

8. Handling Special Cases
8.1 TextEdit — Focus pseudo-state + hint text

TextEdit is the most complex because it has focus state (not just hover/active) and egui's TextEdit already has hint_text(). The wrapper just forwards it, but focus state tracking is critical:
rust

pub fn show(self, ui: &mut Ui) -> Response {
    let id = ui.make_persistent_id("styled_te").with(self.text as *const String);
    let pseudo = PseudoState::load(ui, id);

    let resolved = self.style.resolve(pseudo, /* ... */);

    let response = ui.scope(|ui| {
        // Override visuals...
        // Also override selection color, cursor color if specified

        let mut te = egui::TextEdit::singleline(self.text);
        if let Some(hint) = &self.hint {
            te = te.hint_text(hint.as_str());
        }
        if self.password {
            te = te.password(true);
        }

        // Sizing
        if self.style.full_width {
            te = te.desired_width(f32::INFINITY);
        }

        ui.add(te)
    }).inner;

    PseudoState::from_response(&response).store(ui, id);
    response
}

8.2 ComboBox — Closure-based API

ComboBox doesn't implement Widget. It has its own .show() that takes a closure. We wrap it directly:
rust

pub struct StyledComboBox {
    id_source: Id,
    selected_text: WidgetText,
    style: SharedStyle,
    width: Option<f32>,
}

impl StyledComboBox {
    pub fn show(
        self,
        ui: &mut Ui,
        menu_contents: impl FnOnce(&mut Ui),
    ) -> InnerResponse<Option<()>> {
        // Scoped visuals for the combo button itself
        ui.scope(|ui| {
            // Apply visuals...
            let mut cb = egui::ComboBox::from_id_source(self.id_source)
                .selected_text(self.selected_text);
            if let Some(w) = self.width {
                cb = cb.width(w);
            }
            cb.show_ui(ui, menu_contents)
        }).inner
    }
}

8.3 Slider — Numeric generics

egui's Slider is generic over Numeric. We handle this with a generic parameter:
rust

pub struct StyledSlider<'a, T: egui::emath::Numeric> {
    value: &'a mut T,
    range: RangeInclusive<T>,
    style: SharedStyle,
    // slider-specific
    text: Option<String>,
    step: Option<f64>,
}

9. Helper Functions
rust

// src/helpers.rs

/// Shorthand for Color32::from_rgb
pub const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Shorthand for Color32::from_rgba_premultiplied
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(r, g, b, a)
}

/// Convert f32 into Rounding::same()
impl From<f32> for Rounding {
    // Note: egui already has this, but we may need our own newtype
    // if the conversion isn't available
}

/// Margin from a single f32 (uniform padding)
pub fn uniform_margin(v: f32) -> Margin {
    Margin::same(v)
}

10. Phased Implementation Plan
Phase 1: Minimal viable crate (1-2 days)

Goal: StyledButton and StyledFrame working with bg, hover_bg, active_bg, rounding, border, text_color. No theme system. No pseudo-state (just use egui's built-in hover detection from ui.scope).

Deliverables:

    SharedStyle struct (partial, just bg/border/rounding)
    StyledButton with .show() using ui.scope() + visuals override
    StyledFrame wrapping egui::Frame
    Styled namespace
    rgb() helper
    One example: examples/basic.rs

Why this first: The ui.scope() approach for visuals override is the fundamental technique. If this works cleanly, everything else is iteration.
Phase 2: Pseudo-states + TextEdit (2-3 days)

Goal: Add PseudoState via egui::Memory. Implement StyledTextEdit with focus_border, hover_bg. Add .hint(), .password().

Deliverables:

    state.rs — PseudoState load/store
    Refactor StyledButton to use pseudo-state for active_bg
    StyledTextEdit with full pseudo-state support
    full_width(), margin_top/bottom()
    The impl_shared_style_methods! macro

Phase 3: Containers + Layout (2 days)

Goal: StyledRow, StyledColumn with gap support. Margin/padding fully working.

Deliverables:

    StyledRow and StyledColumn
    Gap support via scoped item_spacing
    Padding via inner_margin on Frame
    Margin via add_space() (top/bottom) — horizontal margin is harder and may need a wrapping approach

Phase 4: Theme system (1-2 days)

Goal: Theme struct, ThemeExt trait, ctx.set_theme(), ctx.theme().

Deliverables:

    theme.rs — full Theme struct with dark/light presets
    ThemeExt on egui::Context
    examples/theme_demo.rs with theme switching

Phase 5: Apply + Style composition (1 day)

Goal: The Apply trait. Ergonomic style functions. Document the pattern.

Deliverables:

    Apply trait, implemented for all styled types
    examples/composable_styles.rs showing reusable style functions
    README section on style composition

Phase 6: Remaining widgets (2-3 days)

Goal: StyledCheckbox, StyledSlider, StyledComboBox, StyledLabel.

Deliverables:

    Each widget with appropriate pseudo-state support
    Widget-specific builder methods (slider step/text, checkbox label color, etc.)

Phase 7: Polish and publish (2 days)

    Full documentation with doc examples
    Performance audit — ensure no allocations in hot paths beyond what egui itself does
    CI, tests (snapshot tests for rendered output if feasible, otherwise interaction tests)
    README.md with comparison to raw egui
    Publish to crates.io

11. Public API Summary for the 6 Core Widgets

Here's what the finished API looks like in practice, showing realistic game UI code:
rust

use egui_styled::{Styled, rgb, ThemeExt};

fn settings_panel(ui: &mut Ui, state: &mut AppState) {
    let theme = ui.ctx().theme();

    Styled::frame()
        .bg(theme.bg_surface)
        .rounding(theme.rounding_lg)
        .padding(theme.spacing_lg)
        .border(1.0, theme.border)
        .show(ui, |ui| {

            // Section header
            Styled::label("Settings")
                .font_size(theme.font_size_lg)
                .text_color(theme.fg_primary)
                .margin_bottom(theme.spacing_md)
                .show(ui);

            // Username input
            Styled::text_edit(&mut state.username)
                .hint("Username")
                .full_width()
                .bg(theme.bg_secondary)
                .hover_bg(theme.bg_elevated)
                .rounding(theme.rounding_md)
                .border(1.0, theme.border)
                .focus_border(1.0, theme.border_focus)
                .margin_bottom(theme.spacing_md)
                .show(ui);

            // Password input
            Styled::text_edit(&mut state.password)
                .hint("Password")
                .password(true)
                .full_width()
                .bg(theme.bg_secondary)
                .rounding(theme.rounding_md)
                .border(1.0, theme.border)
                .focus_border(1.0, theme.border_focus)
                .margin_bottom(theme.spacing_md)
                .show(ui);

            // Volume slider
            Styled::slider(&mut state.volume, 0.0..=1.0)
                .text("Volume")
                .full_width()
                .rounding(theme.rounding_full)
                .margin_bottom(theme.spacing_md)
                .show(ui);

            // Checkbox
            Styled::checkbox(&mut state.vsync, "Enable V-Sync")
                .text_color(theme.fg_secondary)
                .margin_bottom(theme.spacing_lg)
                .show(ui);

            // Button row
            Styled::row().gap(theme.spacing_md).show(ui, |ui| {

                if Styled::button("Save")
                    .apply(|b| primary_button(b, &theme))
                    .show(ui)
                    .clicked()
                {
                    state.save();
                }

                if Styled::button("Cancel")
                    .bg(Color32::TRANSPARENT)
                    .text_color(theme.fg_muted)
                    .hover_bg(theme.bg_elevated)
                    .rounding(theme.rounding_md)
                    .show(ui)
                    .clicked()
                {
                    state.revert();
                }
            });
        });
}

fn primary_button(b: StyledButton, theme: &Theme) -> StyledButton {
    b.bg(theme.accent)
     .hover_bg(theme.accent_hover)
     .active_bg(theme.accent_active)
     .text_color(Color32::WHITE)
     .rounding(theme.rounding_md)
}

12. Performance Notes
Concern	Mitigation
SharedStyle is cloned per widget?	No. Builder consumes self by value, no cloning. Struct is ~200 bytes of Options on the stack.
PseudoState Memory lookups	One get_temp + one insert_temp per styled widget per frame. These are HashMap lookups on TypeId + Id. Nanoseconds.
ui.scope() overhead	Creates a child Ui with a cloned Style (~1KB). egui does this constantly internally. Negligible.
Theme reads from Memory	One hash lookup per ctx.theme() call. Cache it in a local variable at the top of your panel function.
Allocations	SharedStyle is stack-allocated. String for hint text is the only potential heap alloc, and only if you use .hint().

Bottom line: Per styled widget, the overhead vs raw egui is approximately: 1 scope creation + 2 Memory lookups + 1 style resolve (branch chain over Options). This is well under 1μs per widget on any modern CPU.

This gives you a complete, implementable blueprint. Shall I write the actual code for Phase 1 — SharedStyle, StyledButton, StyledFrame, the Styled namespace, and the basic example?