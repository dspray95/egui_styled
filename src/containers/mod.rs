pub mod area;
pub mod collapsing;
pub mod column;
pub mod distribute;
pub mod frame;
pub mod row;
pub mod spacer;
pub mod stack;
pub mod wrap;

use egui::{Id, Response, Ui, UiBuilder, Vec2};

/// One item of a measured row, with the parent's `spacing` restored. The explicit id keeps
/// the item's widget ids identical on measuring and layout passes.
pub(crate) fn row_item(ui: &mut Ui, id: Id, spacing: Vec2, item: impl FnOnce(&mut Ui)) -> Response {
    ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
        ui.spacing_mut().item_spacing = spacing;
        item(ui);
    })
    .response
}
