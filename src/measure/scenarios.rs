//! Pass counts and first-frame correctness for measured containers.

use std::cell::Cell;

use egui::{Context, Id, RawInput, Rect, Ui};

use crate::Styled;

fn screen() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}

/// Runs one frame; returns egui's pass count.
fn frame(context: &Context, ui_fn: impl FnMut(&mut Ui)) -> usize {
    crate::test_util::run_ui(context, screen(), ui_fn)
        .platform_output
        .num_completed_passes
}

fn tag(ui: &mut Ui, text: &str) -> Rect {
    Styled::button(text).show(ui).rect
}

#[test]
fn static_ui_settles_to_one_pass() {
    let context = Context::default();
    let ui_fn = |ui: &mut Ui| {
        Styled::row()
            .full_width()
            .space_between()
            .item(|ui| {
                tag(ui, "A");
            })
            .item(|ui| {
                tag(ui, "B");
            })
            .show(ui);
        Styled::row()
            .full_width()
            .gap(4.0)
            .wrap()
            .item(|ui| {
                tag(ui, "one");
            })
            .item(|ui| {
                tag(ui, "two");
            })
            .show(ui);
    };
    assert_eq!(frame(&context, ui_fn), 2, "first show reruns once");
    for _ in 0..5 {
        assert_eq!(frame(&context, ui_fn), 1);
    }
}

#[test]
fn nested_wrap_in_stack_is_correct_on_first_frame() {
    let context = Context::default();
    let last = Cell::new(Rect::NOTHING);
    let below = Cell::new(Rect::NOTHING);
    let ui_fn = |ui: &mut Ui| {
        ui.vertical_centered(|ui| {
            Styled::stack()
                .layer(|ui| {
                    Styled::row()
                        .gap(4.0)
                        .wrap()
                        .item(|ui| {
                            tag(ui, "alpha");
                        })
                        .item(|ui| {
                            last.set(tag(ui, "beta"));
                        })
                        .show(ui);
                })
                .show(ui);
            below.set(ui.label("below").rect);
        });
    };
    assert_eq!(frame(&context, ui_fn), 2);
    let first = (last.get(), below.get());
    assert_eq!(
        Styled::measure_stats(&context).repaints,
        0,
        "settled in one rerun"
    );
    assert_eq!(frame(&context, ui_fn), 1);
    assert_eq!(
        first,
        (last.get(), below.get()),
        "first frame already settled"
    );
}

#[test]
fn item_appearing_in_distributed_row_reruns_once() {
    let context = Context::default();
    let right = Cell::new(Rect::NOTHING);
    let run = |extra: bool| {
        frame(&context, |ui| {
            let mut row = Styled::row().full_width().space_between().item(|ui| {
                tag(ui, "A");
            });
            if extra {
                row = row.item(|ui| {
                    tag(ui, "extra");
                });
            }
            row.item(|ui| {
                right.set(tag(ui, "B"));
            })
            .show(ui);
        })
    };
    run(false);
    run(false);
    assert_eq!(run(true), 2, "the change is settled in the same frame");
    let edge = right.get().right();
    assert_eq!(run(true), 1);
    assert!(
        (right.get().right() - edge).abs() < 0.5,
        "no shift after the rerun frame"
    );
}

#[test]
fn multipass_off_hides_unmeasured_then_places_correctly() {
    let context = Context::default();
    Styled::set_multipass(&context, false);
    let visible = Cell::new(true);
    let rect = Cell::new(Rect::NOTHING);
    let ui_fn = |ui: &mut Ui| {
        Styled::row()
            .full_width()
            .space_between()
            .item(|ui| {
                tag(ui, "A");
            })
            .item(|ui| {
                visible.set(ui.is_visible());
                rect.set(tag(ui, "B"));
            })
            .show(ui);
    };
    assert_eq!(frame(&context, ui_fn), 1);
    assert!(!visible.get(), "never shown at a guessed position");
    frame(&context, ui_fn);
    assert!(visible.get());
    let rect = rect.get();
    assert!(
        (rect.right() - 800.0).abs() < 10.0,
        "pinned right: {rect:?}"
    );
}

#[test]
fn width_animation_reruns_up_to_max_streak() {
    let context = Context::default();
    let max_streak = Styled::measure_config(&context).max_streak as usize;
    let reruns = (0..60)
        .map(|i| {
            let label = "x".repeat(1 + i % 30);
            frame(&context, |ui| {
                Styled::row()
                    .full_width()
                    .space_between()
                    .item(|ui| {
                        tag(ui, &label);
                    })
                    .item(|ui| {
                        tag(ui, "B");
                    })
                    .show(ui);
            })
        })
        .filter(|&passes| passes == 2)
        .count();
    // First show, then `max_streak` changes before falling back. The animation wraps at 30.
    assert!(reruns <= 1 + 2 * max_streak, "{reruns} rerun frames");
}

#[test]
fn staggered_first_shows_each_rerun_in_their_own_frame() {
    let context = Context::default();
    let differs = |a: &f32, b: &f32| (a - b).abs() > 0.5;

    for i in 0..8usize {
        let passes = frame(&context, |ui| {
            for j in 0..=i {
                super::record(ui.ctx(), Id::unique(("row", j)), "test", 10.0, differs);
            }
        });
        assert_eq!(passes, 2, "new row on frame {i} reruns the frame");
    }
}

#[test]
fn oscillating_measurement_freezes_and_stops_repainting() {
    let context = Context::default();
    let max_changes = Styled::measure_config(&context).max_changes;
    let differs = |a: &f32, b: &f32| (a - b).abs() > 0.5;
    let id = Id::unique("oscillating");
    for i in 0..max_changes + 10 {
        frame(&context, |ui| {
            super::record(ui.ctx(), id, "test", (i % 2) as f32, differs);
        });
    }
    let stats = Styled::measure_stats(&context);
    assert_eq!(stats.freezes, 1);
    assert_eq!(stats.last_freeze, Some(("test", id)));
    let repaints = stats.repaints;
    frame(&context, |ui| {
        super::record(ui.ctx(), id, "test", 5.0, differs)
    });
    assert_eq!(
        Styled::measure_stats(&context).repaints,
        repaints,
        "frozen: no requests"
    );
}

#[test]
fn explicit_id_survives_a_sibling_appearing() {
    let context = Context::default();
    let run = |sibling: bool| {
        frame(&context, |ui| {
            if sibling {
                tag(ui, "new sibling");
            }
            Styled::row()
                .id("stable")
                .full_width()
                .space_between()
                .item(|ui| {
                    tag(ui, "A");
                })
                .item(|ui| {
                    tag(ui, "B");
                })
                .show(ui);
        })
    };
    run(false);
    run(false);
    assert_eq!(run(true), 1, "no remeasure: the cache key didn't move");
}

#[test]
fn distributed_row_in_shrink_frame_in_stack_settles_in_one_rerun() {
    let context = Context::default();
    let below = Cell::new(Rect::NOTHING);
    let ui_fn = |ui: &mut Ui| {
        ui.vertical_centered(|ui| {
            Styled::stack()
                .layer(|ui| {
                    ui.vertical_centered(|ui| {
                        Styled::frame().padding(8.0).show(ui, |ui| {
                            Styled::row()
                                .min_width(240.0)
                                .space_between()
                                .item(|ui| {
                                    tag(ui, "left");
                                })
                                .item(|ui| {
                                    tag(ui, "right");
                                })
                                .show(ui);
                        });
                    });
                })
                .show(ui);
            below.set(ui.label("below").rect);
        });
    };
    assert_eq!(frame(&context, ui_fn), 2);
    let first = below.get();
    assert_eq!(Styled::measure_stats(&context).repaints, 0);
    assert_eq!(frame(&context, ui_fn), 1);
    assert_eq!(first, below.get());
}

#[test]
fn containers_after_a_measured_row_keep_their_ids_across_passes() {
    let context = Context::default();
    let ui_fn = |ui: &mut Ui| {
        Styled::row()
            .full_width()
            .space_between()
            .item(|ui| {
                tag(ui, "A");
            })
            .item(|ui| {
                tag(ui, "B");
            })
            .show(ui);
        Styled::row()
            .full_width()
            .gap(4.0)
            .wrap()
            .item(|ui| {
                tag(ui, "one");
            })
            .show(ui);
        Styled::stack()
            .layer(|ui| {
                ui.label("stack");
            })
            .show(ui);
        ui.vertical_centered(|ui| {
            Styled::frame().show(ui, |ui| {
                ui.label("shrink");
            });
        });
    };
    assert_eq!(frame(&context, ui_fn), 2);
    assert_eq!(
        Styled::measure_stats(&context).repaints,
        0,
        "nothing re-missed in pass 2"
    );
}
