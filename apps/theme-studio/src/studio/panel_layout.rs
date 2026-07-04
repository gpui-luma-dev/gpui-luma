use super::inspectable::InspectableId;

pub const PANEL_WIDTH_PX: f32 = 380.0;
pub const TEAM_PANEL_WIDTH_PX: f32 = 380.0;
pub const ACCOUNT_PANEL_WIDTH_PX: f32 = 340.0;
pub const CHAT_PANEL_WIDTH_PX: f32 = 360.0;
pub const PAYMENTS_PANEL_WIDTH_PX: f32 = 720.0;
pub const TREE_VIEW_PANEL_WIDTH_PX: f32 = 360.0;
pub const ACCORDION_PANEL_WIDTH_PX: f32 = 380.0;
pub const SYSTEM_PREFERENCES_PANEL_WIDTH_PX: f32 = 380.0;
pub const NAVIGATION_SIDEBAR_PANEL_WIDTH_PX: f32 = 300.0;

pub fn panel_width(id: InspectableId) -> f32 {
    match id {
        InspectableId::CreateAccount => ACCOUNT_PANEL_WIDTH_PX,
        InspectableId::Chat => CHAT_PANEL_WIDTH_PX,
        InspectableId::TeamMembers => TEAM_PANEL_WIDTH_PX,
        InspectableId::Payments => PAYMENTS_PANEL_WIDTH_PX,
        InspectableId::TreeView => TREE_VIEW_PANEL_WIDTH_PX,
        InspectableId::Accordion => ACCORDION_PANEL_WIDTH_PX,
        InspectableId::SystemPreferences => SYSTEM_PREFERENCES_PANEL_WIDTH_PX,
        InspectableId::NavigationSidebar => NAVIGATION_SIDEBAR_PANEL_WIDTH_PX,
        _ => PANEL_WIDTH_PX,
    }
}
