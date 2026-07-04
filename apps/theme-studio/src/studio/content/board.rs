use gpui::{IntoElement, div, prelude::*, px};

use super::super::demo_controls::DemoControls;
use super::super::inspectable::InspectableId;
use super::super::panel_layout::panel_width;

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

    div().id("theme-studio-demo-board").flex().flex_wrap().items_start().gap(px(16.0)).children(
        all_panels.into_iter().filter(|(id, _)| filter.is_none_or(|allowed| allowed.contains(id))).map(
            |(id, child)| {
                div().id(format!("panel-{id:?}")).w(px(panel_width(id))).max_w(px(panel_width(id))).child(child)
            },
        ),
    )
}
