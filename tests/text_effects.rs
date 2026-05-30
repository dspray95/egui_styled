use egui::{Align2, Color32, Vec2, vec2};
use egui_kittest::Harness;
use egui_styled::prelude::*;

// ── Unit tests (no rendering) ─────────────────────────────────────────────

#[test]
fn text_effects_is_empty_by_default() {
    assert!(TextEffects::default().is_empty());
}

#[test]
fn text_effects_not_empty_after_shadow() {
    let mut fx = TextEffects::default();
    fx.shadows.push(TextShadow {
        offset: vec2(1.0, 1.0),
        color: Color32::BLACK,
    });
    assert!(!fx.is_empty());
}

#[test]
fn text_effects_not_empty_after_glow() {
    let fx = TextEffects {
        glow: Some(Glow {
            color: Color32::WHITE,
            radius: 8.0,
            intensity: 0.5,
        }),
        ..Default::default()
    };
    assert!(!fx.is_empty());
}

#[test]
fn text_effects_not_empty_after_scale() {
    let fx = TextEffects {
        scale: Some((1.5, Align2::CENTER_CENTER)),
        ..Default::default()
    };
    assert!(!fx.is_empty());
}

#[test]
fn glow_quality_default_values() {
    let q = GlowQuality::default();
    assert_eq!(q.samples, 64);
}

#[test]
fn theme_shadow_tokens_are_present() {
    let theme = StyledTheme::default();
    assert!(theme.shadow_sm != Vec2::ZERO);
    assert!(theme.shadow_md != Vec2::ZERO);
    assert!(theme.shadow_lg != Vec2::ZERO);
}

#[test]
fn theme_glow_tokens_are_positive() {
    let theme = StyledTheme::default();
    assert!(theme.glow_sm > 0.0);
    assert!(theme.glow_md > theme.glow_sm);
    assert!(theme.glow_lg > theme.glow_md);
}

// ── Builder chain smoke-tests (no rendering) ─────────────────────────────

#[test]
fn label_builder_shadow_chains() {
    // Just verifies the chain compiles and doesn't panic before show().
    let _label = Styled::label("test")
        .text_shadow(vec2(1.0, 1.0), Color32::BLACK)
        .text_shadow(vec2(-1.0, 0.0), Color32::BLUE)
        .text_color(Color32::WHITE);
}

#[test]
fn label_builder_outline() {
    let _label = Styled::label("test")
        .outline(1.5, Color32::BLACK)
        .text_color(Color32::WHITE);
}

#[test]
fn label_builder_glow() {
    let _label = Styled::label("test")
        .glow(Color32::from_rgb(0, 220, 255), 8.0, 0.8)
        .glow_quality(32)
        .text_color(Color32::WHITE);
}

#[test]
fn label_builder_scale() {
    let _label = Styled::label("test")
        .scale(1.3, Align2::CENTER_CENTER)
        .text_color(Color32::WHITE);
}

// ── Snapshot tests (require wgpu renderer) ───────────────────────────────
//
// First run generates the baseline images in `tests/snapshots/`.
// Subsequent runs compare against them. Run with:
//   `cargo test` (uses wgpu renderer by default via egui_kittest)
//
// To regenerate snapshots after intentional visual changes:
//   `UPDATE_SNAPSHOTS=true cargo test`

#[test]
fn snapshot_text_shadow() {
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("SHADOW")
            .font(egui::FontId::proportional(24.0))
            .text_color(Color32::WHITE)
            .text_shadow(vec2(2.0, 2.0), Color32::from_black_alpha(180))
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("text_shadow");
}

#[test]
fn snapshot_chromatic_aberration() {
    let cyan = Color32::from_rgb(0, 220, 255);
    let magenta = Color32::from_rgb(255, 0, 200);
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("[ENTER]")
            .font(egui::FontId::proportional(24.0))
            .text_color(Color32::WHITE)
            .text_shadow(vec2(-2.0, 0.0), cyan)
            .text_shadow(vec2(2.0, 0.0), magenta)
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("chromatic_aberration");
}

#[test]
fn snapshot_outline() {
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("OUTLINE")
            .font(egui::FontId::proportional(24.0))
            .text_color(Color32::WHITE)
            .outline(2.0, Color32::BLACK)
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("text_outline");
}

#[test]
fn snapshot_glow() {
    // Fixed intensity for deterministic output — no animation.
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("GLOW")
            .font(egui::FontId::proportional(24.0))
            .text_color(Color32::WHITE)
            .glow(Color32::from_rgb(0, 220, 255), 8.0, 0.7)
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("text_glow");
}

#[test]
fn snapshot_scale() {
    // Fixed factor — animation stays consumer-side.
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("SCALE")
            .font(egui::FontId::proportional(24.0))
            .text_color(Color32::WHITE)
            .scale(1.4, Align2::CENTER_CENTER)
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("text_scale");
}

#[test]
fn snapshot_composed_glow_shadow() {
    let cyan = Color32::from_rgb(0, 220, 255);
    let magenta = Color32::from_rgb(255, 0, 200);
    let mut harness = Harness::new_ui(|ui| {
        Styled::label("GAME OVER")
            .font(egui::FontId::proportional(32.0))
            .text_color(Color32::WHITE)
            .text_shadow(vec2(-1.5, 0.0), cyan)
            .text_shadow(vec2(1.5, 0.0), magenta)
            .glow(cyan, 12.0, 0.6)
            .extend()
            .show(ui);
    });
    harness.run();
    harness.snapshot("composed_glow_shadow");
}
