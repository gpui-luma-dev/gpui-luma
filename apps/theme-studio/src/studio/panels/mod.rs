mod account;
mod chat;
mod common;

pub(crate) use common::{format_hex_color, format_hsla, parse_hex_color};
mod cookies;
mod navigation_sidebar;
mod payments;
mod report;
mod team;
mod upgrade;

pub use account::AccountPanel;
pub use chat::ChatPanel;
pub use cookies::CookiesPanel;
pub use navigation_sidebar::NavigationSidebarPanel;
pub use payments::PaymentsPanel;
pub use report::ReportPanel;
pub use team::TeamPanel;
pub use upgrade::UpgradePanel;

use std::collections::HashMap;

use gpui::{Context, MouseButton, Point, Pixels, div, prelude::*, px};

use gpui_luma::theme::LumaChrome;

use super::demo_controls::DemoControls;
use super::inspectable::InspectableId;
use super::panel_layout::{DemoPanelDrag, PANEL_BOARD_MIN_HEIGHT_PX, default_panel_position};
use super::studio_board::StudioBoardHost;

use self::common::panel_drag_handle;

pub fn render_demo_board(
    selected: Option<InspectableId>,
    positions: HashMap<InspectableId, Point<Pixels>>,
    demos: DemoControls,
    chrome: LumaChrome,
    cx: &mut Context<StudioBoardHost>,
) -> impl IntoElement {
    let pick = |id: InspectableId, child: gpui::AnyElement| {
        let selected_panel = selected == Some(id);
        let position = positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id));
        let drag = DemoPanelDrag::new(id);

        div()
            .absolute()
            .left(position.x)
            .top(position.y)
            .flex()
            .flex_col()
            .rounded(px(12.0))
            .when(selected_panel, |panel| {
                panel.border_2().border_color(chrome.title_text).bg(gpui::hsla(0.0, 0.0, 1.0, 0.05))
            })
            .when(!selected_panel, |panel| panel.border_1().border_color(gpui::hsla(0.0, 0.0, 0.0, 0.0)))
            .child(
                div()
                    .id(format!("panel-drag-{id:?}"))
                    .on_mouse_down(MouseButton::Left, {
                        cx.listener(move |host, event, _, cx| host.begin_panel_drag(id, event, cx))
                    })
                    .on_drag(drag, |drag, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| drag.clone())
                    })
                    .child(panel_drag_handle(chrome)),
            )
            .child(
                div()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |host, _, _, cx| host.select_inspectable(id, cx)),
                    )
                    .child(child),
            )
            .into_any_element()
    };

    div()
        .id("theme-studio-demo-board")
        .relative()
        .w_full()
        .min_h(px(PANEL_BOARD_MIN_HEIGHT_PX))
        .on_drag_move(cx.listener(StudioBoardHost::handle_panel_drag_move))
        .on_mouse_up(MouseButton::Left, cx.listener(StudioBoardHost::end_panel_drag))
        .on_mouse_up_out(MouseButton::Left, cx.listener(StudioBoardHost::end_panel_drag))
        .child(pick(InspectableId::UpgradeSubscription, demos.upgrade.clone().into_any_element()))
        .child(pick(InspectableId::CreateAccount, demos.account.clone().into_any_element()))
        .child(pick(InspectableId::TeamMembers, demos.team.clone().into_any_element()))
        .child(pick(InspectableId::Chat, demos.chat.clone().into_any_element()))
        .child(pick(InspectableId::CookieSettings, demos.cookies.clone().into_any_element()))
        .child(pick(InspectableId::ReportIssue, demos.report.clone().into_any_element()))
        .child(pick(InspectableId::Payments, demos.payments.clone().into_any_element()))
        .child(pick(InspectableId::NavigationSidebar, demos.navigation_sidebar.clone().into_any_element()))
}
