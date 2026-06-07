mod account;
mod chat;
mod common;

pub(crate) use common::{format_hex_color, parse_hex_color};
mod cookies;
mod dashboard;
mod navigation_sidebar;
pub(crate) mod palette;
mod payments;
mod task_list;
mod report;
mod team;
mod tree_view;
mod upgrade;

pub use account::AccountPanel;
pub use chat::ChatPanel;
pub use cookies::CookiesPanel;
pub use dashboard::DashboardPanel;
pub use palette::PalettePanel;
pub use payments::PaymentsPanel;
pub use report::ReportPanel;
pub use team::TeamPanel;
pub use tree_view::TreeViewPanel;
pub use upgrade::UpgradePanel;

use std::collections::HashMap;

use gpui::{Context, MouseButton, Point, Pixels, div, prelude::*, px};

use gpui_luma::theme::LumaChrome;

use super::demo_controls::DemoControls;
use super::inspectable::InspectableId;
use super::panel_layout::{DemoPanelDrag, PANEL_BOARD_MIN_HEIGHT_PX, default_panel_position};
use super::content_pane::ContentPaneHost;

pub fn render_demo_board(
    selected: Option<InspectableId>,
    positions: HashMap<InspectableId, Point<Pixels>>,
    panel_z_order: HashMap<InspectableId, u32>,
    demos: DemoControls,
    chrome: LumaChrome,
    filter: Option<&'static [InspectableId]>,
    cx: &mut Context<ContentPaneHost>,
) -> impl IntoElement {
    let pick = |id: InspectableId, child: gpui::AnyElement| {
        let selected_panel = selected == Some(id);
        let position = positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id));
        let drag = DemoPanelDrag::new(id);

        let border_color = if selected_panel {
            chrome.title_text
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        };

        div()
            .absolute()
            .left(position.x)
            .top(position.y)
            .flex()
            .flex_col()
            .occlude()
            .rounded(px(12.0))
            .border_2()
            .border_color(border_color)
            .when(selected_panel, |panel| panel.bg(gpui::hsla(0.0, 0.0, 1.0, 0.05)))
            .child(
                div()
                    .id(format!("panel-{id:?}"))
                    .cursor_grab()
                    .on_mouse_down(MouseButton::Left, {
                        cx.listener(move |host, event, _, cx| host.begin_panel_drag(id, event, cx))
                    })
                    .on_drag(drag, |drag, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| *drag)
                    })
                    .child(child),
            )
            .into_any_element()
    };

    let all_panels: [(InspectableId, gpui::AnyElement); 8] = [
        (InspectableId::UpgradeSubscription, demos.upgrade.clone().into_any_element()),
        (InspectableId::CreateAccount, demos.account.clone().into_any_element()),
        (InspectableId::TeamMembers, demos.team.clone().into_any_element()),
        (InspectableId::Chat, demos.chat.clone().into_any_element()),
        (InspectableId::CookieSettings, demos.cookies.clone().into_any_element()),
        (InspectableId::ReportIssue, demos.report.clone().into_any_element()),
        (InspectableId::Payments, demos.payments.clone().into_any_element()),
        (InspectableId::TreeView, demos.tree_view.clone().into_any_element()),
    ];

    let mut panels: Vec<_> = all_panels
        .into_iter()
        .filter(|(id, _)| filter.is_none_or(|allowed| allowed.contains(id)))
        .map(|(id, child)| (id, panel_z_order.get(&id).copied().unwrap_or(0), pick(id, child)))
        .collect();
    panels.sort_by_key(|(_, z, _)| *z);

    let mut board = div()
        .id("theme-studio-demo-board")
        .relative()
        .w_full()
        .min_h(px(PANEL_BOARD_MIN_HEIGHT_PX))
        .on_drag_move(cx.listener(ContentPaneHost::handle_panel_drag_move))
        .on_mouse_up(MouseButton::Left, cx.listener(ContentPaneHost::end_panel_drag))
        .on_mouse_up_out(MouseButton::Left, cx.listener(ContentPaneHost::end_panel_drag));

    for (_, _, panel) in panels {
        board = board.child(panel);
    }

    board
}
