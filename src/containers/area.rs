use egui::{Align2, Context, InnerResponse, Order, Pos2, Vec2};

use crate::{
    containers::frame::StyledFrame, impl_style_builders, style::shared_style::SharedStyle,
};

/// Top-level positioned container — modal panels, backdrops, toasts.
///
/// Unlike [`StyledFrame`] / [`StyledRow`] / [`StyledColumn`], `show` takes
/// `&Context` (not `&mut Ui`) because the underlying [`egui::Area`] is a
/// floating layer that doesn't live inside the current `Ui` tree. Use this
/// when you need anchored or screen-relative placement; use `StyledFrame`
/// when you just want a styled box inside a normal layout.
///
/// All [`SharedStyle`] box builders (`bg`, `border`, `padding`,
/// `corner_radius`, `margin`) apply to the inner frame.
pub struct StyledArea {
    id: Option<egui::Id>,
    anchor: Option<(Align2, Vec2)>,
    fixed_pos: Option<Pos2>,
    order: Option<Order>,
    interactable: bool,
    movable: bool,
    fill_screen: bool,
    align: Option<egui::Align>,
    style: SharedStyle,
}

impl Default for StyledArea {
    fn default() -> Self {
        Self::new()
    }
}

impl StyledArea {
    pub fn new() -> Self {
        Self {
            id: None,
            anchor: None,
            fixed_pos: None,
            order: None,
            interactable: true,
            movable: false,
            fill_screen: false,
            align: None,
            style: SharedStyle::default(),
        }
    }

    /// Stable id for this area. Defaults to one derived from call-site
    /// auto-id; set explicitly if you have multiple areas of the same shape.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Some(egui::Id::new(id));
        self
    }

    /// Anchor to a fixed screen position (e.g., `Align2::CENTER_CENTER`).
    pub fn anchor(mut self, align: Align2, offset: Vec2) -> Self {
        self.anchor = Some((align, offset));
        self
    }

    /// Place at a fixed screen-space position. Wins over `anchor` if both set.
    pub fn fixed_pos(mut self, pos: Pos2) -> Self {
        self.fixed_pos = Some(pos);
        self
    }

    /// Set the egui rendering order (Background / Middle / Foreground / Tooltip / etc).
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    pub fn interactable(mut self, interactable: bool) -> Self {
        self.interactable = interactable;
        self
    }

    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    /// Stretch the area to cover the full screen. Useful for backdrops,
    /// dim layers, and full-bleed overlays. Overrides anchor/fixed_pos.
    pub fn fill_screen(mut self) -> Self {
        self.fill_screen = true;
        self
    }

    /// Cross-axis alignment forwarded to the inner styled frame.
    pub fn align(mut self, align: egui::Align) -> Self {
        self.align = Some(align);
        self
    }

    pub fn show<R>(self, ctx: &Context, body: impl FnOnce(&mut egui::Ui) -> R) -> InnerResponse<R> {
        let id = self.id.unwrap_or_else(|| egui::Id::new("styled_area"));
        let mut area = egui::Area::new(id)
            .interactable(self.interactable)
            .movable(self.movable);
        if let Some(order) = self.order {
            area = area.order(order);
        }
        if self.fill_screen {
            area = area.fixed_pos(ctx.content_rect().min);
        } else if let Some(pos) = self.fixed_pos {
            area = area.fixed_pos(pos);
        } else if let Some((align, offset)) = self.anchor {
            area = area.anchor(align, offset);
        }

        let frame = StyledFrame {
            style: self.style,
            align: self.align,
        };
        let fill_screen = self.fill_screen;
        let screen_size = ctx.content_rect().size();

        area.show(ctx, |ui| {
            if fill_screen {
                ui.set_min_size(screen_size);
            }
            frame.show(ui, body).inner
        })
    }
}

impl_style_builders!(StyledArea);
