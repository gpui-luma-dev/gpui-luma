use std::collections::HashMap;

use gpui::{Context, Empty, IntoElement, Pixels, Point, Render, Window, point, px};

use super::inspectable::InspectableId;

pub const PANEL_WIDTH_PX: f32 = 380.0;
pub const TEAM_PANEL_WIDTH_PX: f32 = 380.0;
pub const ACCOUNT_PANEL_WIDTH_PX: f32 = 340.0;
pub const CHAT_PANEL_WIDTH_PX: f32 = 360.0;
pub const PAYMENTS_PANEL_WIDTH_PX: f32 = 720.0;
pub const TREE_VIEW_PANEL_WIDTH_PX: f32 = 360.0;
pub const ACCORDION_PANEL_WIDTH_PX: f32 = 380.0;
pub const NAVIGATION_SIDEBAR_PANEL_WIDTH_PX: f32 = 300.0;

pub fn panel_width(id: InspectableId) -> f32 {
    match id {
        InspectableId::CreateAccount => ACCOUNT_PANEL_WIDTH_PX,
        InspectableId::Chat => CHAT_PANEL_WIDTH_PX,
        InspectableId::TeamMembers => TEAM_PANEL_WIDTH_PX,
        InspectableId::Payments => PAYMENTS_PANEL_WIDTH_PX,
        InspectableId::TreeView => TREE_VIEW_PANEL_WIDTH_PX,
        InspectableId::Accordion => ACCORDION_PANEL_WIDTH_PX,
        InspectableId::NavigationSidebar => NAVIGATION_SIDEBAR_PANEL_WIDTH_PX,
        _ => PANEL_WIDTH_PX,
    }
}
const PANEL_GAP_PX: f32 = 16.0;
const PANEL_ROW_HEIGHT_PX: f32 = 440.0;
pub const PANEL_BOARD_MIN_HEIGHT_PX: f32 = PANEL_ROW_HEIGHT_PX * 3.0 + PANEL_GAP_PX * 2.0 + 80.0;

#[derive(Clone, Copy)]
pub struct DemoPanelDrag {
    pub id: InspectableId,
}

impl DemoPanelDrag {
    pub fn new(id: InspectableId) -> Self {
        Self { id }
    }
}

impl Render for DemoPanelDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub fn default_panel_positions() -> HashMap<InspectableId, Point<Pixels>> {
    let col_step = PANEL_WIDTH_PX + PANEL_GAP_PX;
    InspectableId::all()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, id)| {
            let col = (index % 3) as f32;
            let row = (index / 3) as f32;
            (id, point(px(col * col_step), px(row * PANEL_ROW_HEIGHT_PX)))
        })
        .collect()
}

pub fn default_panel_position(id: InspectableId) -> Point<Pixels> {
    default_panel_positions().get(&id).copied().unwrap_or_else(|| point(px(0.0), px(0.0)))
}

pub fn offset_panel_position(origin: Point<Pixels>, delta: Point<Pixels>) -> Point<Pixels> {
    point(px(origin.x.as_f32() + delta.x.as_f32()), px(origin.y.as_f32() + delta.y.as_f32()))
}
