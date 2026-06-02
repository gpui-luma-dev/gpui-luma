use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, Context, Div, FocusHandle, Pixels, Stateful, Window, div, prelude::*, px};

use super::{
    control::{ResizablePanels, ResizablePanelsHandleDrag},
    model::{ResizablePanelSpec, ResizablePanelsOrientation, ResizablePanelsRenderModel},
    theme::ResizablePanelsAppearance,
};

pub const MIN_HIDDEN_HANDLE_HIT_TARGET_PX: f32 = 12.0;

pub trait ResizablePanelsTemplate: Send + Sync {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        appearance: &ResizablePanelsAppearance,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div>;
}

pub struct ThemedResizablePanelsTemplate;

impl ThemedResizablePanelsTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThemedResizablePanelsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_resizable_panels_template() -> Arc<dyn ResizablePanelsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ResizablePanelsTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedResizablePanelsTemplate)).clone()
}

impl ResizablePanelsTemplate for ThemedResizablePanelsTemplate {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        appearance: &ResizablePanelsAppearance,
        handle_focuses: &[FocusHandle],
        _window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div> {
        let panel_count = model.panels.len();
        let effective_handle_visual_size = model.handle_size.max(px(1.0));
        let effective_handle_interaction_size = if model.show_handle {
            effective_handle_visual_size
        } else {
            effective_handle_visual_size.max(px(MIN_HIDDEN_HANDLE_HIT_TARGET_PX))
        };

        let total_main = match model.orientation {
            ResizablePanelsOrientation::Horizontal => model.frame_width.unwrap_or(px(1.0)),
            ResizablePanelsOrientation::Vertical => model.frame_height.unwrap_or(px(1.0)),
        }
        .max(px(1.0));

        let total_handle = if panel_count > 1 {
            effective_handle_visual_size * ((panel_count - 1) as f32)
        } else {
            px(0.0)
        };
        let content_main = (total_main - total_handle).max(px(1.0));

        let mut root = div()
            .id(model.id.clone())
            .overflow_hidden()
            .on_drag_move(cx.listener(ResizablePanels::handle_drag_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(ResizablePanels::finish_drag))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(ResizablePanels::finish_drag));

        root = match (model.frame_width, model.frame_height) {
            (Some(width), Some(height)) => root.w(width).h(height),
            (Some(width), None) => root.w(width).h_full(),
            (None, Some(height)) => root.h(height).w_full(),
            (None, None) => root.size_full(),
        };

        if model.show_border {
            root = root.border_1().border_color(appearance.border);
        }

        let mut track = div().size_full().flex();
        if model.orientation == ResizablePanelsOrientation::Vertical {
            track = track.flex_col();
        }

        for (index, panel) in model.panels.iter().enumerate() {
            let size = model.sizes.get(index).unwrap_or(&0.0);
            let clamped = size.clamp(panel.min_size, panel.max_size);
            let main_size = content_main * (clamped / 100.0);

            track = track.child(render_panel(model.orientation, panel, main_size));

            if index + 1 < panel_count {
                track = track.child(render_handle_divider(
                    index,
                    model,
                    appearance,
                    &handle_focuses[index],
                    effective_handle_visual_size,
                    effective_handle_interaction_size,
                    cx,
                ));
            }
        }

        root.child(track)
    }
}

fn render_panel(
    orientation: ResizablePanelsOrientation,
    panel: &ResizablePanelSpec,
    main_size: Pixels,
) -> impl IntoElement {
    let panel_node = div().overflow_hidden().child((panel.render.clone())());
    match orientation {
        ResizablePanelsOrientation::Horizontal => panel_node.w(main_size).h_full(),
        ResizablePanelsOrientation::Vertical => panel_node.h(main_size).w_full(),
    }
}

fn render_handle_divider(
    index: usize,
    model: &ResizablePanelsRenderModel<'_>,
    appearance: &ResizablePanelsAppearance,
    focus: &FocusHandle,
    handle_visual_size: Pixels,
    handle_interaction_size: Pixels,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let enabled = model.enabled;
    let orientation = model.orientation;
    let handle_id = format!("{}-handle-{index}", model.id);

    let mut handle = div()
        .id(handle_id)
        .track_focus(focus)
        .tab_index(if enabled { 0 } else { -1 })
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .when(!enabled, |this| this.opacity(appearance.disabled_opacity))
        .when(enabled && orientation == ResizablePanelsOrientation::Horizontal, |this| this.cursor_col_resize())
        .on_key_down(cx.listener(move |this, event, window, cx| {
            this.handle_handle_key_down(index, event, window, cx);
        }));

    if orientation == ResizablePanelsOrientation::Horizontal {
        handle = handle.w(handle_visual_size).h_full();
    } else {
        handle = handle.h(handle_visual_size).w_full().when(enabled, |this| this.cursor_row_resize());
    }

    let drag_payload = ResizablePanelsHandleDrag { id: model.id.clone(), handle_index: index };
    let interaction_layer = render_handle_interaction_layer(
        index,
        model.id.clone(),
        orientation,
        handle_visual_size,
        handle_interaction_size,
        enabled,
        drag_payload,
        cx,
    );

    if model.show_handle {
        let grip_color = if model.handle_grip {
            appearance.grip_emphasis
        } else {
            appearance.grip
        };

        let mut divider = div().absolute().bg(appearance.divider);
        divider = if orientation == ResizablePanelsOrientation::Horizontal {
            let center_x = (handle_interaction_size.as_f32() - 1.0) * 0.5;
            divider.left(px(center_x)).top(px(0.0)).bottom(px(0.0)).w(px(1.0))
        } else {
            let center_y = (handle_interaction_size.as_f32() - 1.0) * 0.5;
            divider.top(px(center_y)).left(px(0.0)).right(px(0.0)).h(px(1.0))
        };

        let grip = if orientation == ResizablePanelsOrientation::Horizontal {
            div().rounded(px(8.0)).bg(grip_color).w(px(4.0)).h(px(40.0))
        } else {
            div().rounded(px(8.0)).bg(grip_color).w(px(40.0)).h(px(4.0))
        };

        handle
            .child(div().relative().size_full().flex().items_center().justify_center().child(divider).child(grip))
            .child(interaction_layer)
            .into_any_element()
    } else {
        let visual = if orientation == ResizablePanelsOrientation::Horizontal {
            div().w(handle_visual_size.min(px(1.0))).h_full().bg(appearance.divider)
        } else {
            div().h(handle_visual_size.min(px(1.0))).w_full().bg(appearance.divider)
        };

        let anchor = if orientation == ResizablePanelsOrientation::Horizontal {
            div().size_full().flex().items_stretch().justify_end().child(visual)
        } else {
            div().size_full().flex().flex_col().items_stretch().justify_end().child(visual)
        };

        handle.child(anchor).child(interaction_layer).into_any_element()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_handle_interaction_layer(
    index: usize,
    id: gpui::SharedString,
    orientation: ResizablePanelsOrientation,
    handle_visual_size: Pixels,
    handle_interaction_size: Pixels,
    enabled: bool,
    drag_payload: ResizablePanelsHandleDrag,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let hit_id = format!("{id}-handle-hit-{index}");

    if orientation == ResizablePanelsOrientation::Horizontal {
        let left = (handle_visual_size.as_f32() - handle_interaction_size.as_f32()) * 0.5;
        div()
            .id(hit_id)
            .absolute()
            .left(px(left))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(handle_interaction_size)
            .when(enabled, |this| this.cursor_col_resize())
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.handle_handle_mouse_down(index, event, window, cx);
                }),
            )
            .on_drag(drag_payload.clone(), {
                let drag = drag_payload.clone();
                move |_: &ResizablePanelsHandleDrag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                }
            })
            .into_any_element()
    } else {
        let top = (handle_visual_size.as_f32() - handle_interaction_size.as_f32()) * 0.5;
        div()
            .id(hit_id)
            .absolute()
            .top(px(top))
            .left(px(0.0))
            .right(px(0.0))
            .h(handle_interaction_size)
            .when(enabled, |this| this.cursor_row_resize())
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event, window, cx| {
                    this.handle_handle_mouse_down(index, event, window, cx);
                }),
            )
            .on_drag(drag_payload, |drag: &ResizablePanelsHandleDrag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .into_any_element()
    }
}
