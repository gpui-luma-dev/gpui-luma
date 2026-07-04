use gpui::{IntoElement, div, prelude::*};

use super::super::demo_controls::DemoControls;
use super::super::inspectable::InspectableId;
use super::super::panel_layout::{GAP, panel_height_class};
use super::super::prototypes::column_layout::{ColumnLayout, ColumnPlacementStrategy, ColumnTile};

pub fn render_demo_board(demos: DemoControls, filter: Option<&'static [InspectableId]>) -> impl IntoElement {
    let all_panels: [(InspectableId, gpui::AnyElement); 12] = [
        (InspectableId::UpgradeSubscription, demos.upgrade.clone().into_any_element()),
        (InspectableId::CreateAccount, demos.account.clone().into_any_element()),
        (InspectableId::TeamMembers, demos.team.clone().into_any_element()),
        (InspectableId::Chat, demos.chat.clone().into_any_element()),
        (InspectableId::CookieSettings, demos.cookies.clone().into_any_element()),
        (InspectableId::ReportIssue, demos.report.clone().into_any_element()),
        (InspectableId::Payments, demos.payments.clone().into_any_element()),
        (InspectableId::ShareDocument, demos.share.clone().into_any_element()),
        (InspectableId::DatePickerRange, demos.date_range.clone().into_any_element()),
        (InspectableId::TreeView, demos.tree_view.clone().into_any_element()),
        (InspectableId::Accordion, demos.accordion.clone().into_any_element()),
        (InspectableId::SystemPreferences, demos.system_preferences.clone().into_any_element()),
    ];

    ColumnLayout::new()
        .columns(3)
        .gap_x(GAP)
        .gap_y(GAP)
        .strategy(ColumnPlacementStrategy::GreedyByEstimatedHeight)
        .tiles(all_panels.into_iter().filter(|(id, _)| filter.is_none_or(|allowed| allowed.contains(id))).map(
            |(id, child)| {
                ColumnTile::new(div().id(format!("panel-{id:?}")).w_full().child(child))
                    .height_class(panel_height_class(id))
            },
        ))
}
