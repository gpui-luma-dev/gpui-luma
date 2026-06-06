use gpui::{Context, DragMoveEvent, Entity, MouseDownEvent, MouseUpEvent, Render, Window, div, prelude::*, px};

use super::app::ThemeStudioApp;
use super::inspectable::InspectableId;
use super::panel_layout::{DemoPanelDrag, PANEL_BOARD_MIN_HEIGHT_PX};
use super::panels::render_demo_board;

pub struct StudioBoardHost {
    app: Entity<ThemeStudioApp>,
}

impl StudioBoardHost {
    pub fn new(app: Entity<ThemeStudioApp>) -> Self {
        Self { app }
    }

    pub fn begin_panel_drag(&mut self, id: InspectableId, event: &MouseDownEvent, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| app.begin_panel_drag(id, event, cx));
    }

    pub fn handle_panel_drag_move(
        &mut self,
        event: &DragMoveEvent<DemoPanelDrag>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.app.update(cx, |app, cx| app.handle_panel_drag_move(event, window, cx));
    }

    pub fn end_panel_drag(&mut self, _event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.app.update(cx, |app, cx| app.end_panel_drag(window, cx));
    }
}

impl Render for StudioBoardHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let app = self.app.read(cx);
        let chrome = app.look.chrome();
        let board_bg = app.look.token_color("background").unwrap_or(chrome.app_background);

        div()
            .size_full()
            .min_h(px(PANEL_BOARD_MIN_HEIGHT_PX))
            .bg(board_bg)
            .p(px(24.0))
            .child(render_demo_board(
                app.selected,
                app.panel_positions.clone(),
                app.panel_z_order.clone(),
                app.demos.clone(),
                chrome,
                cx,
            ))
    }
}
