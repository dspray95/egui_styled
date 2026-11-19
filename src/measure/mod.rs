//! Measurement for containers that lay out from their content's size.
//! A change reruns the pass (`request_discard`), or repaints when a rerun isn't allowed.

mod config;
mod debug;
mod measured;
#[cfg(test)]
mod scenarios;
mod stats;

pub use config::MeasureConfig;
pub use stats::MeasureStats;

use egui::{Context, Id};
use measured::Measured;

#[derive(Clone, Debug, Default)]
struct State {
    config: MeasureConfig,
    stats: MeasureStats,
    force_multipass: bool,
}

fn with_state<R>(context: &Context, f: impl FnOnce(&mut State) -> R) -> R {
    context.data_mut(|d| f(d.get_temp_mut_or_default(Id::unique("egui_styled::measure"))))
}

pub(crate) fn config(context: &Context) -> MeasureConfig {
    with_state(context, |s| s.config)
}

pub(crate) fn set_config(context: &Context, config: MeasureConfig) {
    with_state(context, |s| s.config = config);
}

pub(crate) fn stats(context: &Context) -> MeasureStats {
    with_state(context, |s| s.stats.clone())
}

pub(crate) fn force_multipass(context: &Context, on: bool) {
    with_state(context, |s| s.force_multipass = on);
    context.add_plugin(debug::ForceMultipass);
}

pub(crate) fn cached<T: Clone + Send + Sync + 'static>(context: &Context, id: Id) -> Option<T> {
    let value = context
        .data(|d| d.get_temp::<Measured<T>>(id))
        .and_then(|m| m.value);
    if value.is_none() {
        with_state(context, |s| s.stats.misses += 1);
    }
    value
}

pub(crate) fn record<T: Clone + Send + Sync + 'static>(
    context: &Context,
    id: Id,
    kind: &'static str,
    new: T,
    changed: impl Fn(&T, &T) -> bool,
) {
    let frame_nr = context.cumulative_frame_nr();
    let streak = context.data_mut(|d| {
        d.get_temp_mut_or_default::<Measured<T>>(id)
            .update(new, frame_nr, changed)
    });
    if let Some(streak) = streak {
        settle(context, id, kind, streak);
    }
}

/// Report a miss that has no value to record, e.g. egui's own invisible first-show pass.
pub(crate) fn miss(context: &Context, id: Id, kind: &'static str) {
    with_state(context, |s| s.stats.misses += 1);
    settle(context, id, kind, 0);
}

enum Request {
    Rerun,
    Repaint,
    Nothing,
}

fn settle(context: &Context, id: Id, kind: &'static str, streak: u32) {
    let request = with_state(context, |s| {
        let c = s.config;
        if streak > c.max_changes {
            if streak == c.max_changes + 1 {
                s.stats.freezes += 1;
                s.stats.last_freeze = Some((kind, id));
                log::warn!("egui_styled: {kind} {id:?} keeps changing size, frozen");
            }
            Request::Nothing
        } else if c.multipass && streak <= c.max_streak {
            Request::Rerun
        } else {
            Request::Repaint
        }
    });

    // Outside `with_state`: calling the context inside `data_mut` deadlocks.
    match request {
        Request::Nothing => {}
        Request::Rerun => {
            context.request_discard(format!("egui_styled {kind} {id:?} measurement changed"));
            if context.will_discard() {
                with_state(context, |s| s.stats.discards += 1);
            } else {
                with_state(context, |s| s.stats.repaints += 1);
                context.request_repaint();
            }
        }
        Request::Repaint => {
            with_state(context, |s| s.stats.repaints += 1);
            context.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn differs(a: &f32, b: &f32) -> bool {
        (a - b).abs() > 0.5
    }

    /// Runs one frame that records `value`; returns how many passes egui ran.
    fn frame(context: &Context, value: f32) -> usize {
        let id = Id::unique("test_container");
        crate::test_util::run_ui(context, egui::RawInput::default(), |ui| {
            record(ui.ctx(), id, "test", value, differs);
        })
        .platform_output
        .num_completed_passes
    }

    #[test]
    fn first_measurement_reruns_once_then_settles() {
        let context = Context::default();
        assert_eq!(frame(&context, 10.0), 2);
        assert_eq!(frame(&context, 10.0), 1);
    }

    #[test]
    fn animation_reruns_up_to_max_streak_then_repaints() {
        let context = Context::default();
        assert_eq!(frame(&context, 10.0), 2, "first measurement");
        assert_eq!(frame(&context, 20.0), 2, "streak 1");
        assert_eq!(
            frame(&context, 30.0),
            1,
            "streak 2 > max_streak 1: fallback"
        );
        let stats = with_state(&context, |s| s.stats.clone());
        assert_eq!((stats.discards, stats.repaints), (2, 1));
    }

    #[test]
    fn multipass_off_never_reruns() {
        let context = Context::default();
        with_state(&context, |s| s.config.multipass = false);
        assert_eq!(frame(&context, 10.0), 1);
        assert_eq!(with_state(&context, |s| s.stats.repaints), 1);
    }

    #[test]
    fn force_multipass_reruns_every_frame_until_turned_off() {
        let context = Context::default();
        let passes = |context: &Context| {
            crate::test_util::run_ui(context, egui::RawInput::default(), |_| {})
                .platform_output
                .num_completed_passes
        };
        assert_eq!(passes(&context), 1);
        force_multipass(&context, true);
        assert_eq!(passes(&context), 2);
        assert_eq!(passes(&context), 2);
        force_multipass(&context, false);
        assert_eq!(passes(&context), 1);
    }
}
