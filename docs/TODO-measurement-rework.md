# TODO: measurement rework, remaining work

Implemented: `src/measure/` (cross-pass measurement with rerun, streak, budget and freeze
guards), ported wrap, distribute, justify, shrink-to-fit, stack and centred area, `.id()`
builders, README and CHANGELOG. See the README's Measurement section for behaviour.

## Pick the default (`MeasureConfig::multipass`, provisionally on)

`cargo run --release --example measure_bench [frames] [rounds]` times a busy headless screen
(UI passes plus tessellation) in static, discrete-change and animated scenarios, on and off.
Pin glibc's allocator when running it (see the example's header): otherwise heap trimming
refaults ~250 KB of pages every frame, costs ~25%, and varies with first-frame history.

Pinned results (2000 frames x 7 rounds, µs): static 615 vs `HEAD` 603; discrete changes 644
on vs 618 off (rerun frames cost ~1.85x); animated 642 vs `HEAD` 1151 (old shrink-to-fit
reran every frame). On and off are identical when nothing changes.

- Run with `Styled::debug_force_multipass` on to flush out UI code that isn't pass-safe.

## Visual checks

`cargo run --example measure_check` has a card per check below, plus live tuning and stats.

- Size animation inside a measured container at `max_streak = 1` (one-frame-lag fallback
  after the first change). If the lag shows, raise `max_streak` and let the budget cap cost.
- Freeze: a self-dependent distributed row (full-width item) under input, and a size
  animation longer than `max_changes` that ends at a new size.
- Staggered appearance (list items one per frame) once the budget is spent: refused items
  appear one frame late.

## Tuning

`max_streak`, `max_changes` and `budget` defaults (1, 180, 4 of 16) are first guesses.
Revisit with the stats from real screens.

## Known gaps

- Natural size isn't layout-aware: items that wrap text or fill width can measure
  differently on the miss pass, costing an extra frame. `.extend()` on labels avoids it.
- Measured containers need the same `Ui` id structure on measuring and layout passes.
  Rows and stacks pin child ids explicitly; keep that invariant when adding containers.
- `StyledLabel::scale` scales the text but not the label's own `bg`, so a scaled label
  with a background overflows it. Pre-existing; visible since stack layers stopped stretching.
