use gpui::{Context, Hsla, MouseButton, Render, Window, WindowControlArea, div, prelude::*, px};
use gpui_luma::shell::{TITLE_BAR_HEIGHT, TITLE_BAR_LEFT_PADDING};

struct TitleBarDragState {
    should_move: bool,
}

impl Render for TitleBarDragState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
    }
}

pub struct SplitColumn {
    background: Hsla,
    traffic_light_inset: bool,
}

impl SplitColumn {
    pub fn new(background: Hsla, traffic_light_inset: bool) -> Self {
        Self { background, traffic_light_inset }
    }
}

impl Render for SplitColumn {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let background = self.background;
        let traffic_light_inset = self.traffic_light_inset;
        let state = window.use_state(cx, |_, _| TitleBarDragState { should_move: false });

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .id(if traffic_light_inset {
                        "split-column-title-left"
                    } else {
                        "split-column-title-right"
                    })
                    .h(TITLE_BAR_HEIGHT)
                    .flex_shrink_0()
                    .w_full()
                    .when(traffic_light_inset, |title| title.pl(TITLE_BAR_LEFT_PADDING))
                    .bg(background)
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
                        state.should_move = false;
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = true;
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        window.listener_for(&state, |state, _, _, _| {
                            state.should_move = false;
                        }),
                    )
                    .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                        if state.should_move {
                            state.should_move = false;
                            window.start_window_move();
                        }
                    }))
                    .on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                        if ev.click_count >= 2 {
                            cx.stop_propagation();
                            if !window.is_fullscreen() {
                                window.zoom_window();
                            }
                        }
                    }),
            )
            .child(div().flex_1().min_h(px(0.0)).bg(background))
    }
}
