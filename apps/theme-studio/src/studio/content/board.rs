use std::collections::HashMap;

use gpui::{Context, MouseButton, Point, Pixels, div, prelude::*, px};

use super::host::ContentPaneHost;
use super::super::demo_controls::DemoControls;
use super::super::inspectable::InspectableId;
use super::super::panel_layout::{DemoPanelDrag, PANEL_BOARD_MIN_HEIGHT_PX, default_panel_position};

pub fn render_demo_board(
    _selected: Option<InspectableId>,
    positions: HashMap<InspectableId, Point<Pixels>>,
    panel_z_order: HashMap<InspectableId, u32>,
    demos: DemoControls,
    filter: Option<&'static [InspectableId]>,
    cx: &mut Context<ContentPaneHost>,
) -> impl IntoElement {
    let pick = |id: InspectableId, child: gpui::AnyElement| {
        let position = positions.get(&id).copied().unwrap_or_else(|| default_panel_position(id));
        let drag = DemoPanelDrag::new(id);

        div()
            .absolute()
            .left(position.x)
            .top(position.y)
            .flex()
            .flex_col()
            .occlude()
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

    let all_panels: [(InspectableId, gpui::AnyElement); 9] = [
        (InspectableId::UpgradeSubscription, demos.upgrade.clone().into_any_element()),
        (InspectableId::CreateAccount, demos.account.clone().into_any_element()),
        (InspectableId::TeamMembers, demos.team.clone().into_any_element()),
        (InspectableId::Chat, demos.chat.clone().into_any_element()),
        (InspectableId::CookieSettings, demos.cookies.clone().into_any_element()),
        (InspectableId::ReportIssue, demos.report.clone().into_any_element()),
        (InspectableId::Payments, demos.payments.clone().into_any_element()),
        (InspectableId::TreeView, demos.tree_view.clone().into_any_element()),
        (InspectableId::Accordion, demos.accordion.clone().into_any_element()),
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
