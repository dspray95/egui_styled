# TODO: measurement rework (multi-pass, stale caches, stack hit-testing)

Status: designed, not started. Pick up during proper game UI design work.

## Problems

1. **Stale measurements.** `StyledWrappingRow` (`containers/wrap.rs`) caches per-item widths and only
   re-measures when the item count changes. `distribute_row_horizontally` (`style/shared_style.rs`)
   caches total content width and never re-measures. Content that changes size (live numbers, a
   conditionally shown button) keeps the old layout forever. `justify_body_vertically` writes back
   every frame but never requests a repaint, so in a reactive app it stays wrong until the next input.
2. **Invisible first frame.** A cache miss renders invisibly and calls `request_repaint()`, so newly
   shown content is blank for one frame. `StyledArea::fixed_pos_centered` (`containers/area.rs`) is
   also blank on first show (egui's own `Area` sizing pass), and keeps a redundant size cache.
3. **Stack hit-testing.** `StyledStack` paints at a provisional origin, then translates its shapes
   (`PaintList::transform_range`). Widget hit rects stay at the provisional origin, so interactive
   layers are clickable in the wrong place. The stack also allocates its own `Response` after its
   children, and egui hit-tests the last-registered widget as topmost, so a stack with
   `.sense(Sense::click())` likely swallows its children's clicks.
4. **Cache identity.** Cache keys use `next_auto_id()`, which shifts when UI is shown conditionally,
   so a container can read another container's measurement. Unnamed `StyledArea`s all fall back to
   `Id::unique("styled_area")`, so they share egui area state and the centred-size cache.

## Design

### Shared measurement helper

One module used by wrap, distribute, justify, stack and the centred area. Each container stores one
record in egui memory under its cache id:

```rust
struct Measured<T> {
    value: Option<T>,        // None = never measured, distinct from a real zero
    streak: u8,              // consecutive frames with a change
    last_change_frame: u64,  // ctx.cumulative_frame_nr() at the last change
}
```

Two calls: `cached(ctx, id) -> Option<T>` to lay out with, and `record(ctx, id, measured)` after
drawing. Generic over a "changed beyond epsilon" comparison so it covers `f32`, `Vec2` and `Vec<f32>`.

`None -> Some` is the first change and sets `streak = 1` directly. Don't derive it from
`last_change_frame`: its default of 0 equals frame 0 and would leave the streak at 0.

### Per-pass flow

Every container, every pass:

1. Read the cached value.
2. `None`: render in a `UiBuilder::new().invisible().sizing_pass()` child, then allocate the
   **predicted final size** computed from the measurement (not the measuring layout's size).
3. `Some`: lay out visibly from the cached value.
4. Measure what was actually drawn and `record` it.

There is no dedicated measuring frame. Every pass measures; only a never-measured container renders
invisibly.

Allocating the predicted final size on a miss is what lets nested measured containers (a wrap row in
a stack) settle in two passes: the parent measures the child's real size in pass 1.

Only render invisibly on a miss if multi-pass is opted in (see Pass safety) **and** another pass is
possible (`ctx.current_pass_index() + 1 < max_passes`). Otherwise render visibly at the best guess
(the measuring layout itself) and repaint: a one-frame pop beats a blank frame. egui's default
`max_passes` is 2, so the opt-in check is needed: without it, opt-in-off apps would render invisibly
with no discard, which is today's blank frame.

`sizing_pass()` is not a rename of `set_invisible()`: fill and justified widgets shrink to natural
size in it. That is what wrap rows want, but a container whose content fills will measure
differently on the visible pass. See Remaining caveats.

### On a change

When `record` sees a value change beyond epsilon, it stores it, then:

- **Discard** (`ctx.request_discard(reason)`) if multi-pass is enabled, the container's streak is
  at most `MAX_STREAK`, and the global budget is not spent. egui reruns the UI in the same frame, so
  the stale layout is never shown.
- **Otherwise `request_repaint()`.** The next frame uses the new value: a one-frame lag with the
  content visible throughout.
- If a discard was requested but `ctx.will_discard()` is false (pass cap reached), also repaint.
- **Freeze** after `MAX_CHANGES` consecutive changing frames: keep the current value, stop
  repainting for it, and `log::warn!` once. Resume on the next quiet frame. Without this, a
  measurement that depends on its own output (a full-width item in a distributed row: its width
  sets the gap, the gap sets its width) repaints every frame forever. Today's write-once cache can't
  loop; the new design can.

**Per-container streak.** Counts consecutive *frames* with a change, keyed on
`cumulative_frame_nr()`, never per pass:

```rust
m.streak = match frame - m.last_change_frame {
    0 => m.streak,                    // later pass, same frame
    1 => m.streak.saturating_add(1),  // changed last frame too
    _ => 1,                           // first change after a quiet frame
};
```

Never reset the streak on "no change": pass 2 of every discard frame sees no change, so that would
reset it every frame and the guard would never trip. A quiet frame resets it implicitly.

**Global budget.** At most K discard frames per N-frame window across the context, in one memory
slot. Catches staggered changes (list items animating in one frame apart), where every container has
streak 1. K/N is the worst-case extra UI cost over the window, e.g. 4/16 caps it at 25%.

### Per container

| Container | Cached value | Notes |
|---|---|---|
| Wrap row | `Vec<f32>` item widths, plus heights for the miss path | Measures each item's own scope rect on the real layout. Miss path allocates the wrapped size from `assign_lines` and per-line max heights. A changed item count is treated as a miss: index-matching widths is wrong once items are inserted. |
| Distributed row | `f32` sum of item widths | Excludes the injected gaps. Miss path allocates available width by row height. |
| Justify (frame, area) | `f32` content height | Miss path allocates the fill height. |
| Stack | `Vec2` union size | `allocate_exact_size(cached)` **before** painting, then paint layers at `final_rect.min`. Drop `transform_range`. Do not move the stack to its own transformed layer: that reintroduces the z-order and layer-collision problems. |
| Centred area | none | Not ported. Use egui's `Area::pivot(Align2::CENTER_CENTER).fixed_pos(center)` and delete `__centered_size`. egui's own sizing pass handles first show but only repaints; request a discard when the area has no `AreaState` yet. |

### Cache identity

Add an explicit `id_salt` builder on each measured container; fall back to the positional id. With
re-measuring on every pass, a positional collision now costs a discard or a one-frame lag instead
of a permanently wrong layout.

Also covers the background-image fade id (`containers/frame.rs`, also `next_auto_id`): a shifted id
restarts the fade. Unnamed `StyledArea`s fall back to a call-site or positional id instead of the
shared `"styled_area"`.

## Pass safety

On a discard frame, user UI code runs once per pass. egui discards what pass 1 *drew*, not what it
*did*: writes to app state persist into pass 2. Code that reads back what it writes (toggles,
`+= dt`, pushing events) applies twice. egui's own input is safe because pass 1 consumes all events;
engine-level input read inside the UI closure is not.

Example (citizen-engine `debug_panel`): `if just_pressed(F3) { panel.0 = !panel.0 }` runs inside
`run_ui`. On a two-pass frame, both passes see F3 pressed and the panel flips twice. Fixing that is
the engine's job; egui_styled's job is the following:

1. **Opt-in.** Multi-pass is off by default (`Styled::set_multipass(ctx, true)` or similar). Off
   means always taking the repaint path, which still fixes problems 1, 3 and 4. The opt-in doc
   comment states the rule in one line and links a docs section with the F3 example.
2. **`Styled::debug_force_multipass(ctx, bool)`.** Registers an egui plugin (`Context::add_plugin`,
   `Plugin::on_begin_pass`) that requests a discard when `current_pass_index() == 0`. Works because
   `run_ui` clears discard reasons before `begin_pass`, not after. Turns intermittent
   pass-safety bugs in user code into deterministic ones. Dev only.
3. **Discard log.** Every discard egui_styled requests logs at `log::debug!` with container kind,
   id, old and new value, and whether it was granted or fell back. The reason string passed to
   `request_discard` names the container, so egui's own
   "request_discard has been called N frames in a row" warning points at it too.
4. **Audit our own writes.** Everything egui_styled stores in memory must be idempotent per pass.
   Pseudo-state store (a set) and the background-image fade start (get-or-insert) already are; the
   streak counter is per frame by design.

## Verified egui 0.37 behaviour

- `Context::run_ui` loops while a discard was requested, up to `Options::max_passes` (default 2).
  Past the cap, further discards are ignored and the last pass is shown.
- `RawInput::take` moves input events into the first pass; later passes see no events, so egui clicks
  fire once, in the first pass, hit-tested against the previous frame's final rects.
- Several containers requesting a discard in one pass share a single extra pass.
- egui itself already discards for `Grid` (first show), `Resize` (sizing pass) and `Aligned` (size
  change), so apps can hit multi-pass today without egui_styled.
- Hit testing picks the last-registered widget as topmost (`hit_test.rs`).

## Tuning

- **Pass-count tests.** `FullOutput::platform_output.num_completed_passes` gives passes per frame.
  Expected counts:
  - static UI: 1 every frame;
  - first show of a wrap row nested in a stack: 2, then 1;
  - button appearing in a distributed row: 2 once, then 1;
  - 60-frame width animation: at most `MAX_STREAK` frames at 2, the rest at 1;
  - 20 list items staggered by one frame: total extra passes within the global budget.
- **`MAX_STREAK`.** A one-off change needs one discard, so 1 is the principled minimum. Raise only if
  real UIs show legitimate two-frame settles (e.g. a texture landing the frame after first show).
- **Global budget.** Pick the acceptable overhead and derive K and N from it.
- **Real screens.** Run game screens (transitions, popups, HUD with live values) with the discard
  log on. Unexpected discards usually mean a container wants a fixed footprint (`layer_fixed`,
  `min_width`, tabular figures).
- **`MAX_CHANGES`.** Well above `MAX_STREAK` so real size animations finish, e.g. a few seconds of
  frames. Only exists to stop non-converging measurements.
- Make `MAX_STREAK`, `MAX_CHANGES` and the budget configurable, with these defaults.

## Remaining caveats

- **Frame-time spikes.** A discard frame costs about twice a normal one. Total work over two frames
  is unchanged, but it lands in one. Bounded by the guards, and only on discrete changes.
- **Hover on first show.** Pass 2 hit-tests against pass 1's rects from the invisible, disabled
  measuring pass. First show only; pseudo-state already lags a frame.
- **Natural size is not layout-aware.** Items that wrap text or fill width measure differently in
  the sizing pass than in the real layout. Unchanged from today; `.extend()` on labels remains the
  advice. Where the predicted size is off, nesting can still take an extra frame to settle.
  `sizing_pass()` makes the mismatch more likely for fill content: on first show it costs a discard
  plus a repaint frame.
- **Size animations inside measured containers** discard for `MAX_STREAK` frames, then run with a
  one-frame lag. Paint-only animations (fades, `transform_range` scaling) never trigger a change.
- **Behaviour changes.** Stack click order flips (children above the stack's own response). Tests
  that assume "frame 1 measures, frame 2 lays out" need rewriting to a single `run_ui`.

## Done when

- README "Limitations" no longer lists the measurement-frame entry, and the "Note on interactivity
  in stacks" and "Interactive widgets inside translated stacks" entries are gone.
- README documents the multi-pass opt-in, the pass-safety rule, and `debug_force_multipass`.
- Regression tests:
  - content width change re-lays out a distributed and a wrapping row;
  - a button inside a centred stack is clickable at its painted position, including with
    `.sense(Sense::click())` on the stack;
  - with multi-pass on: a first-shown measured container is visible and correctly placed on its
    first frame, and a nested wrap-in-stack is correct on its first frame;
  - with multi-pass off: a first-shown container is visible (best guess) on its first frame;
  - a self-dependent measurement (full-width item in a distributed row) freezes and stops
    requesting repaints;
  - two unnamed centred `StyledArea`s don't share state;
  - the pass-count scenarios above;
  - the library's examples render identically with `debug_force_multipass` on and off.
