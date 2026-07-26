use std::sync::Arc;

use gpui::{
    AnyElement, App, Anchor, KeyDownEvent, MouseButton, MouseDownEvent, Size, Window, anchored, div, point, prelude::*,
    px, transparent_black,
};
use gpui_luma::controls::resizable_panels::{ResizeHandleSize, ResizablePanelsLook};
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::ShadcnLook;

use super::handle::{
    SlidePanelResizeHandlers, SlidePanelResizeHitHandlers, render_slide_panel_inner_resize_handle,
    render_slide_panel_resize_handle,
};
use super::state::{SUPPORTED_TOP_ANCHORS, SlidePanelEdge, SlidePanelState};

const DEFAULT_SIDE_PANEL_WIDTH: f32 = 360.0;
const EDGE_PANEL_DESIRED_HEIGHT: f32 = 420.0;
const EDGE_PANEL_MAX_VIEWPORT_RATIO: f32 = 0.72;

pub(crate) type SlidePanelKeyDownHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
pub(crate) type SlidePanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

pub(crate) struct SlidePanelOverlayHandlers {
    pub key_down: SlidePanelKeyDownHandler,
    pub backdrop_mouse_down: SlidePanelMouseDownHandler,
}

pub(crate) fn render_slide_panel_overlay(
    look: &Arc<ShadcnLook>,
    panels_look: &ResizablePanelsLook,
    state: &SlidePanelState,
    viewport: Size<gpui::Pixels>,
    panel_content: AnyElement,
    handlers: SlidePanelOverlayHandlers,
    resize_handlers: SlidePanelResizeHandlers,
) -> AnyElement {
    debug_assert_eq!(SUPPORTED_TOP_ANCHORS.len(), 2);

    let Some(edge) = state.active_edge() else {
        return div().into_any_element();
    };

    let chrome = look.chrome();
    let panel_background = look.token_color("card").unwrap_or(chrome.panel_background);
    let overlay_top_inset = state.top_anchor().inset();
    let available_height = (viewport.height - overlay_top_inset).max(px(1.0));
    let viewport_width = viewport.width.as_f32();
    let viewport_height = available_height.as_f32();

    let capped_edge_panel_height = EDGE_PANEL_DESIRED_HEIGHT.min(viewport_height * EDGE_PANEL_MAX_VIEWPORT_RATIO);
    let side_panel_width = state.main_axis_size(DEFAULT_SIDE_PANEL_WIDTH);
    let edge_panel_height = state.main_axis_size(capped_edge_panel_height);
    let (panel_width, panel_height) = match edge {
        SlidePanelEdge::Left | SlidePanelEdge::Right => (px(side_panel_width), available_height),
        SlidePanelEdge::Top | SlidePanelEdge::Bottom => (viewport.width, px(edge_panel_height)),
    };

    let horizontal_offset = px(-((1.0 - state.open_progress()) * side_panel_width));
    let vertical_offset = px(-((1.0 - state.open_progress()) * edge_panel_height));
    let split_px = resize_split_px(
        edge,
        viewport_width,
        viewport_height,
        side_panel_width,
        edge_panel_height,
        horizontal_offset,
        vertical_offset,
    );

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
        .on_key_down(handlers.key_down)
        .child(panel_content);

    let handle_metrics = ResizeHandleSize::Sm.metrics();
    let handle_active = state.resize_handle_hovered() || state.is_resizing();
    let SlidePanelResizeHandlers { mouse_down, drag_move, mouse_up, mouse_up_out, hover } = resize_handlers;
    let resize_handle = render_slide_panel_resize_handle(
        edge,
        panels_look,
        &handle_metrics,
        split_px,
        handle_active,
        SlidePanelResizeHitHandlers { mouse_down, hover },
    );

    let mut overlay_root = div()
        .id("slide-panel-window-overlay")
        .relative()
        .w(viewport.width)
        .h(available_height)
        .overflow_hidden()
        .on_drag_move(drag_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out);

    if state.backdrop_click_closes() {
        overlay_root = overlay_root.child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .bg(transparent_black())
                .on_mouse_down(MouseButton::Left, handlers.backdrop_mouse_down),
        );
    }

    anchored()
        .snap_to_window_with_margin(px(0.0))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), overlay_top_inset))
        .child(overlay_root.child(panel_shell).child(resize_handle))
        .into_any_element()
}

pub(crate) fn render_slide_panel_inset(
    panels_look: &ResizablePanelsLook,
    state: &SlidePanelState,
    panel_content: AnyElement,
    panel_background: gpui::Hsla,
    handlers: SlidePanelOverlayHandlers,
    resize_handlers: SlidePanelResizeHandlers,
) -> AnyElement {
    let Some(edge) = state.active_edge() else {
        return div().into_any_element();
    };

    debug_assert_eq!(edge, SlidePanelEdge::Right, "button pane inset panel supports right edge only");

    let side_panel_width = state.main_axis_size(DEFAULT_SIDE_PANEL_WIDTH);
    let panel_width = px(side_panel_width);
    let horizontal_offset = px(-((1.0 - state.open_progress()) * side_panel_width));

    let handle_metrics = ResizeHandleSize::Sm.metrics();
    let handle_active = state.resize_handle_hovered() || state.is_resizing();
    let SlidePanelResizeHandlers { mouse_down, drag_move, mouse_up, mouse_up_out, hover } = resize_handlers;
    let resize_handle = render_slide_panel_inner_resize_handle(
        edge,
        panels_look,
        &handle_metrics,
        handle_active,
        SlidePanelResizeHitHandlers { mouse_down, hover },
    );

    let panel_shell = div()
        .id("slide-panel-shell")
        .absolute()
        .occlude()
        .right(horizontal_offset)
        .top_0()
        .bottom_0()
        .w(panel_width)
        .border_l_1()
        .border_color(panels_look.divider)
        .bg(panel_background)
        .on_key_down(handlers.key_down)
        .child(panel_content)
        .child(resize_handle);

    let mut overlay_root = div()
        .id("slide-panel-inset-overlay")
        .absolute()
        .inset_0()
        .overflow_hidden()
        .on_drag_move(drag_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out);

    if state.backdrop_click_closes() {
        overlay_root = overlay_root.child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .bg(transparent_black())
                .on_mouse_down(MouseButton::Left, handlers.backdrop_mouse_down),
        );
    }

    overlay_root.child(panel_shell).into_any_element()
}

fn resize_split_px(
    edge: SlidePanelEdge,
    viewport_width: f32,
    viewport_height: f32,
    side_panel_width: f32,
    edge_panel_height: f32,
    horizontal_offset: gpui::Pixels,
    vertical_offset: gpui::Pixels,
) -> f32 {
    match edge {
        SlidePanelEdge::Right => viewport_width - side_panel_width - horizontal_offset.as_f32(),
        SlidePanelEdge::Left => side_panel_width + horizontal_offset.as_f32(),
        SlidePanelEdge::Bottom => viewport_height - edge_panel_height - vertical_offset.as_f32(),
        SlidePanelEdge::Top => edge_panel_height + vertical_offset.as_f32(),
    }
}

pub(crate) fn slide_panel_panels_look(look: &Arc<ShadcnLook>) -> ResizablePanelsLook {
    look.resizable_panels_theme().resolve(InteractionState::default())
}
