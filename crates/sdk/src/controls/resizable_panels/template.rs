use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, Context, Div, FocusHandle, Hsla, Pixels, Stateful, Window, div, prelude::*, px};

use super::{
    control::{ResizablePanels, ResizablePanelsHandleDrag},
    math::{handle_hit_target_main_axis_px, handle_overlay_geometry, split_positions_px},
    model::{
        ResizeHandleMetrics, ResizeHandleVisibility, ResizablePanelSpec, ResizablePanelsOrientation,
        ResizablePanelsRenderModel,
    },
    theme::ResizablePanelsLook,
};

/// Back-compat alias for [`super::math::MIN_HANDLE_LANE_PX`].
pub const MIN_HIDDEN_HANDLE_HIT_TARGET_PX: f32 = super::math::MIN_HANDLE_LANE_PX;

pub type ResizablePanelsTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ResizablePanelsTemplate: Send + Sync {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div>;
}

pub struct ThemedResizablePanelsTemplate {
    modifiers: Vec<ResizablePanelsTemplateModifier>,
}

impl ThemedResizablePanelsTemplate {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ResizablePanelsRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl Default for ThemedResizablePanelsTemplate {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_resizable_panels_template() -> Arc<dyn ResizablePanelsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ResizablePanelsTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedResizablePanelsTemplate::new())).clone()
}

struct ModifiedResizablePanelsTemplate {
    base: Arc<dyn ResizablePanelsTemplate>,
    modifiers: Vec<ResizablePanelsTemplateModifier>,
}

impl ModifiedResizablePanelsTemplate {
    fn new(base: Arc<dyn ResizablePanelsTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ResizablePanelsTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ResizablePanelsRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub(super) fn modified_resizable_panels_template<F>(
    template: Arc<dyn ResizablePanelsTemplate>,
    modifier: F,
) -> Arc<dyn ResizablePanelsTemplate>
where
    F: Fn(Stateful<Div>, &ResizablePanelsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedResizablePanelsTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl ResizablePanelsTemplate for ModifiedResizablePanelsTemplate {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div> {
        let root = self.base.render(model, look, handle_focuses, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl ResizablePanelsTemplate for ThemedResizablePanelsTemplate {
    fn render(
        &self,
        model: &ResizablePanelsRenderModel<'_>,
        look: &ResizablePanelsLook,
        handle_focuses: &[FocusHandle],
        window: &mut Window,
        cx: &mut Context<ResizablePanels>,
    ) -> Stateful<Div> {
        let handle_metrics = model.resize_handle.metrics();
        let split_positions = split_positions_px(model.panel_sizes_px);

        let mut root = div()
            .on_children_prepainted({
                let entity = cx.entity().clone();
                move |bounds, _window, cx| {
                    let Some(size) = bounds.iter().map(|b| b.size).reduce(|acc, next| gpui::Size {
                        width: acc.width.max(next.width),
                        height: acc.height.max(next.height),
                    }) else {
                        return;
                    };
                    entity.update(cx, |this, cx| {
                        this.set_measured_size(size, cx);
                    });
                }
            })
            .id(model.id.clone())
            .relative()
            .overflow_hidden()
            .flex()
            .flex_col()
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
            root = root.border_1().border_color(look.border);
        }

        let mut track = div().relative().flex_1().min_h_0().min_w_0().w_full();
        let mut panels_row = div().w_full().h_full().flex().items_stretch();
        if model.orientation == ResizablePanelsOrientation::Vertical {
            panels_row = panels_row.flex_col();
        }

        for (index, panel) in model.panels.iter().enumerate() {
            let main_axis_px = model.panel_sizes_px.get(index).copied().unwrap_or(0.0);
            let main_size = px(main_axis_px.max(0.0));
            panels_row = panels_row.child(render_panel(model.orientation, panel, main_size, look));
        }

        track = track.child(panels_row);

        for (index, &split_px) in split_positions.iter().enumerate() {
            track = track.child(render_overlay_handle(
                index,
                model,
                look,
                &handle_focuses[index],
                &handle_metrics,
                split_px,
                window,
                cx,
            ));
        }

        let root = root.child(track);
        self.apply_modifiers(root, model)
    }
}

fn render_panel(
    orientation: ResizablePanelsOrientation,
    panel: &ResizablePanelSpec,
    main_size: Pixels,
    look: &ResizablePanelsLook,
) -> impl IntoElement {
    let mut panel_node = div().overflow_hidden().child((panel.render.clone())());
    if let Some(background) = panel.background {
        panel_node = panel_node.bg(background);
    } else {
        panel_node = panel_node.bg(look.border);
    }
    match orientation {
        ResizablePanelsOrientation::Horizontal => panel_node.w(main_size).h_full().flex_shrink_0(),
        ResizablePanelsOrientation::Vertical => panel_node.h(main_size).w_full().flex_shrink_0(),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_overlay_handle(
    index: usize,
    model: &ResizablePanelsRenderModel<'_>,
    look: &ResizablePanelsLook,
    focus: &FocusHandle,
    handle_metrics: &ResizeHandleMetrics,
    split_px: f32,
    window: &mut Window,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let enabled = model.enabled;
    let orientation = model.orientation;
    let handle_id = format!("{}-handle-{index}", model.id);
    let handle_active = model.hovered_handle == Some(index)
        || model.dragging_handle == Some(index)
        || (enabled && focus.is_focused(window));
    let show_handle = match model.handle_visibility {
        ResizeHandleVisibility::Always => true,
        ResizeHandleVisibility::Hover => handle_active,
        ResizeHandleVisibility::Hidden => false,
    };
    let lane_px = if show_handle {
        handle_metrics.lane_px.max(1.0)
    } else {
        1.0
    };
    let (origin_px, lane_px, divider_local_px) = handle_overlay_geometry(split_px, lane_px);

    let mut handle = div()
        .id(handle_id)
        .track_focus(focus)
        .tab_index(if enabled { 0 } else { -1 })
        .absolute()
        .when(!enabled, |this| this.opacity(look.disabled_opacity))
        .when(enabled && orientation == ResizablePanelsOrientation::Horizontal, |this| this.cursor_col_resize())
        .on_key_down(cx.listener(move |this, event, window, cx| {
            this.handle_handle_key_down(index, event, window, cx);
        }));

    handle = match orientation {
        ResizablePanelsOrientation::Horizontal => {
            handle.left(px(origin_px)).top(px(0.0)).bottom(px(0.0)).w(px(lane_px))
        }
        ResizablePanelsOrientation::Vertical => handle
            .top(px(origin_px))
            .left(px(0.0))
            .right(px(0.0))
            .h(px(lane_px))
            .when(enabled, |this| this.cursor_row_resize()),
    };

    let drag_payload = ResizablePanelsHandleDrag { id: model.id.clone(), handle_index: index };
    let handle_hit_px = handle_hit_target_main_axis_px(handle_metrics);
    let interaction_layer = render_handle_interaction_layer(
        index,
        model.id.clone(),
        orientation,
        px(handle_hit_px),
        split_px,
        origin_px,
        enabled,
        drag_payload,
        cx,
    );

    let grip_color = if model.handle_grip {
        look.grip_emphasis
    } else {
        look.grip
    };

    let divider = match orientation {
        ResizablePanelsOrientation::Horizontal => {
            div().absolute().left(px(divider_local_px)).top(px(0.0)).bottom(px(0.0)).w(px(1.0)).bg(look.divider)
        }
        ResizablePanelsOrientation::Vertical => {
            div().absolute().top(px(divider_local_px)).left(px(0.0)).right(px(0.0)).h(px(1.0)).bg(look.divider)
        }
    };

    if show_handle {
        handle = handle.child(render_handle_grip(orientation, grip_color, handle_metrics));
    }

    handle.child(divider).child(interaction_layer).into_any_element()
}

fn render_handle_grip(orientation: ResizablePanelsOrientation, grip_color: Hsla, metrics: &ResizeHandleMetrics) -> Div {
    let cross = metrics.grip_cross_axis_px;
    let main = metrics.grip_main_axis_px;
    div()
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(0.0))
        .bottom(px(0.0))
        .when(orientation == ResizablePanelsOrientation::Horizontal, |this| {
            this.flex().justify_center().items_center()
        })
        .when(orientation == ResizablePanelsOrientation::Vertical, |this| {
            this.flex().flex_col().justify_center().items_center()
        })
        .child(
            div()
                .rounded(px(8.0))
                .bg(grip_color)
                .when(orientation == ResizablePanelsOrientation::Horizontal, |this| this.w(px(cross)).h(px(main)))
                .when(orientation == ResizablePanelsOrientation::Vertical, |this| this.w(px(main)).h(px(cross))),
        )
}

#[allow(clippy::too_many_arguments)]
fn render_handle_interaction_layer(
    index: usize,
    id: gpui::SharedString,
    orientation: ResizablePanelsOrientation,
    handle_hit: Pixels,
    split_px: f32,
    handle_origin_px: f32,
    enabled: bool,
    drag_payload: ResizablePanelsHandleDrag,
    cx: &mut Context<ResizablePanels>,
) -> AnyElement {
    let hit_id = format!("{id}-handle-hit-{index}");
    let hit_px = handle_hit.as_f32();
    let split_local = split_px - handle_origin_px;
    let hit_inset = split_local - hit_px * 0.5;

    if orientation == ResizablePanelsOrientation::Horizontal {
        div()
            .id(hit_id)
            .absolute()
            .left(px(hit_inset))
            .top(px(0.0))
            .bottom(px(0.0))
            .w(handle_hit)
            .when(enabled, |this| this.cursor_col_resize())
            .on_hover(cx.listener(move |this, hovered, _window, cx| {
                this.handle_handle_hover(index, hovered, cx);
            }))
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
        div()
            .id(hit_id)
            .absolute()
            .top(px(hit_inset))
            .left(px(0.0))
            .right(px(0.0))
            .h(handle_hit)
            .when(enabled, |this| this.cursor_row_resize())
            .on_hover(cx.listener(move |this, hovered, _window, cx| {
                this.handle_handle_hover(index, hovered, cx);
            }))
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
