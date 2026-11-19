# TODO: coverage gaps (states, widgets, responsive)

Status: none started. Backlog of things missing from the styling surface, ranked
roughly by how much they protect the core promise ("style in terms of how it
looks", no dropping back to raw egui visuals mid-UI).

Note on overlap: `selected` state and `show_selectable` already exist on
`StyledButton`, and tabs were deliberately declined. See TODO-missing-pieces.md.

## States

1. **`disabled` variant.** First-class `hover_`/`focus_`/`active_` builders exist on
   every widget, but there is no `disabled_` set. egui only offers `ui.disable()`,
   which greys everything uniformly. Add `disabled_bg`, `disabled_text_color`,
   `disabled_border`, etc., plus a `.disabled(bool)` toggle. Highest priority: it
   is the one state Tailwind-style users will expect and the only one we omit.

2. **`selected`/`checked` variant beyond buttons.** Buttons have it; checkbox and
   combo items do not. Extend the selected styling path to `StyledCheckbox` and to
   combo-box items so a checked/selected control can restyle.

## State transitions

3. **`.transition(duration)` easing between states.** Colors currently snap on
   hover/focus/active. Only the background-image reveal animates today
   (`bgimg_fade_alpha`). Drive hover/focus color interpolation with
   `ctx.animate_value_with_time`. Signature Tailwind feature; nice-to-have, not a
   correctness gap.

## Widget coverage (egui core we do not wrap yet)

4. **RadioButton / radio group, and a toggle switch.** Obvious omissions next to
   `StyledCheckbox`. Toggle switch is not an egui core widget but fits the brand.
5. **SelectableLabel / segmented control.** The selectable-button row covers simple
   cases; a dedicated segmented control would be cleaner.
6. **ProgressBar and a spinner / loading indicator.**
7. **Hyperlink / link styling.**
8. **DragValue.** Slider's cousin for precise numeric entry.
9. **Menus / context menu / popup / dropdown, and tooltip styling.** Illusion-breaking:
   a styled app falls back to raw egui visuals the moment a menu or tooltip opens.
10. **Styled scrollbars (`ScrollArea`).** Same problem: a styled frame scrolls with
    default egui scrollbars.

## Utility primitives (not egui widgets, built on frame + label)

11. **Badge / chip / pill, and a divider helper.** Cheap to build, rounds out the
    design-system feel.

## Responsive

12. **Breakpoint utilities (web-first).** Let builders vary values by available
    width, Tailwind `sm:`/`md:`/`lg:` style. Target web primarily; mobile later.
    Needs a design decision on where the breakpoint context lives (ctx-level vs
    per-container available width) before any API.
