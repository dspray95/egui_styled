# TODO: missing pieces found building citizen-engine's dev tools

Status: all resolved. Each gap below was worked around in citizen-engine
(`crates/citizen-engine/src/dev_tools/`), so none blocks anything today.

## Gaps

1. **Done: draggable `StyledArea`.** `default_pos(pos)` and `pivot(align)` seed the position, so
   `movable(true)` works. Id-less areas now get call-site auto ids instead of sharing one.
2. **No tabs. Won't add as a container.** egui has no tab widget either; tab systems live in
   third-party crates (`egui_dock`, `egui_tiles`). A container would have to pick selection storage,
   bar look and placement for the caller. Covered instead by gap 3: a row of
   `StyledButton::show_selectable` calls plus a `match`.
3. **Done: selected state on `StyledButton`.** `selected(bool)`, `selected_bg`, `selected_text_color`,
   and `show_selectable(ui, &mut current, value)` for the `selectable_value` pattern.
4. **Done: `StyledSlider::logarithmic`**, plus `smallest_positive`. The rail painter ports egui's
   log mapping so the handle stays under the pointer.
5. **Done: `StyledSlider::suffix`.** Passed through to the value box's `DragValue`.
6. **Done: `StyledCollapsing`** (`Styled::collapsing`). Wraps `egui::CollapsingHeader`; style
   builders apply to the header only, with `default_open`, `indent`, `icon` and `id`.
