use gpui::{
    AnyElement, App, Corner, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, Size, Window, anchored, div, point,
    prelude::*, px,
};
use gpui_luma_look_shadcn::ShadcnLook;

use super::state::{SUPPORTED_TOP_ANCHORS, SlidePanelEdge, SlidePanelState};

const BACKDROP_ALPHA_MAX: f32 = 0.48;
const SIDE_PANEL_WIDTH: f32 = 360.0;
const EDGE_PANEL_DESIRED_HEIGHT: f32 = 420.0;
const EDGE_PANEL_MAX_VIEWPORT_RATIO: f32 = 0.72;

pub(in crate::gallery) type SlidePanelKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub(in crate::gallery) type SlidePanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

pub(in crate::gallery) struct SlidePanelOverlayHandlers {
    pub key_down: SlidePanelKeyDownHandler,
    pub backdrop_mouse_down: SlidePanelMouseDownHandler,
}

pub(in crate::gallery) fn render_slide_panel_overlay(
    look: &ShadcnLook,
    state: &SlidePanelState,
    viewport: Size<gpui::Pixels>,
    panel_content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
) -> AnyElement {
    debug_assert_eq!(SUPPORTED_TOP_ANCHORS.len(), 2);

    let Some(edge) = state.active_edge() else {
        return div().into_any_element();
    };

    let chrome = look.chrome();
    let border = look.token_color("border").unwrap_or(chrome.border);
    let panel_background = look.token_color("card").unwrap_or(chrome.panel_background);
    let backdrop = Hsla { a: BACKDROP_ALPHA_MAX * state.open_progress(), ..gpui::black() };
    let overlay_top_inset = state.top_anchor().inset();
    let available_height = (viewport.height - overlay_top_inset).max(px(1.0));

    let capped_edge_panel_height =
        px(EDGE_PANEL_DESIRED_HEIGHT.min(available_height.as_f32() * EDGE_PANEL_MAX_VIEWPORT_RATIO));
    let (panel_width, panel_height) = match edge {
        SlidePanelEdge::Left | SlidePanelEdge::Right => (px(SIDE_PANEL_WIDTH), available_height),
        SlidePanelEdge::Top | SlidePanelEdge::Bottom => (viewport.width, capped_edge_panel_height),
    };

    let horizontal_offset = px(-((1.0 - state.open_progress()) * SIDE_PANEL_WIDTH));
    let vertical_offset = px(-((1.0 - state.open_progress()) * panel_height.as_f32()));

    let panel_shell = div()
        .id("slide-panel-shell")
        .absolute()
        .occlude()
        .when(matches!(edge, SlidePanelEdge::Left), |panel| {
            panel.left(horizontal_offset).top_0().bottom_0().w(panel_width)
        })
        .when(matches!(edge, SlidePanelEdge::Right), |panel| {
            panel.right(horizontal_offset).top_0().bottom_0().w(panel_width)
        })
        .when(matches!(edge, SlidePanelEdge::Top), |panel| {
            panel.top(vertical_offset).left_0().right_0().h(panel_height)
        })
        .when(matches!(edge, SlidePanelEdge::Bottom), |panel| {
            panel.bottom(vertical_offset).left_0().right_0().h(panel_height)
        })
        .bg(panel_background)
        .border_1()
        .border_color(border)
        .shadow(vec![gpui::BoxShadow {
            offset: point(px(0.0), px(18.0)),
            blur_radius: px(42.0),
            spread_radius: px(-18.0),
            color: Hsla { a: 0.22 * state.open_progress(), ..gpui::black() },
        }])
        .on_key_down(handlers.key_down)
        .child(panel_content);

    anchored()
        .snap_to_window_with_margin(px(0.0))
        .anchor(Corner::TopLeft)
        .position(point(px(0.0), overlay_top_inset))
        .child(
            div()
                .id("slide-panel-window-overlay")
                .relative()
                .w(viewport.width)
                .h(available_height)
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .h_full()
                        .bg(backdrop)
                        .on_mouse_down(MouseButton::Left, handlers.backdrop_mouse_down),
                )
                .child(panel_shell),
        )
        .into_any_element()
}
