//! Radix listbox composition over persistent SDK state, input, and scrolling.

use std::hash::Hash;
use gpui::{App, Context, SharedString, Div, ElementId, Stateful, Window, div, prelude::*, px};
use gpui_luma::controls::listbox::{
    ListBoxControl, ListBoxFlow, ListBoxItemRenderModel, ListBoxItemState, ListBoxItemTemplate, ListBoxVirtualization,
    make_listbox_item_template,
};
use crate::{Look, ScaleFamily, SemanticRole};
use crate::look::resolve_look;

/// Radix row surface. The SDK binding attaches selection and keyboard input.
pub fn listbox_row(
    look: &Look,
    id: impl Into<ElementId>,
    state: ListBoxItemState,
    focus_visible: bool,
) -> Stateful<Div> {
    let step = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
    div()
        .id(id)
        .rounded(px(4.0))
        .border_1()
        .border_color(if state.active && focus_visible {
            look.resolve_role(SemanticRole::Focus).hsla()
        } else {
            gpui::transparent_black()
        })
        .bg(if state.selected {
            step(4)
        } else {
            gpui::transparent_black()
        })
        .text_color(
            look.resolve_role(if state.enabled {
                SemanticRole::Foreground
            } else {
                SemanticRole::MutedForeground
            })
            .hsla(),
        )
        .font_family(crate::typography::font_family(look))
        .text_sm()
        .px_2()
        .flex()
        .items_center()
        .when(state.enabled, |row| {
            row.cursor_pointer().hover(|style| style.bg(step(5))).active(|style| style.bg(step(6)))
        })
}

/// Default label content for a Radix listbox. The builder supplies the themed
/// selection surface; callers can also provide any SDK item-content template.
pub fn listbox_template<T: 'static>(label: impl Fn(&T) -> SharedString + 'static) -> ListBoxItemTemplate<'static, T> {
    make_listbox_item_template(move |model: &ListBoxItemRenderModel<'_, T>, _: &mut App| div().child(label(model.item)))
}

/// Borrowed list builder. Keep the SDK control on the host between renders;
/// building does not reset focus, selection, or scroll position.
pub struct ListBoxBuilder<'a, M: 'static, T, K: 'static> {
    id: &'static str,
    control: &'a mut ListBoxControl<M, T, K>,
    look: Option<Look>,
    flow: ListBoxFlow,
    template: ListBoxItemTemplate<'a, T>,
}
impl<'a, M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxBuilder<'a, M, T, K> {
    pub fn new(
        id: &'static str,
        control: &'a mut ListBoxControl<M, T, K>,
        template: ListBoxItemTemplate<'a, T>,
    ) -> Self {
        Self {
            id,
            control,
            look: None,
            flow: ListBoxFlow::Vertical { visible_items: 5, item_height: 36.0, gap: 4.0 },
            template,
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn flow(mut self, flow: ListBoxFlow) -> Self {
        self.flow = flow;
        self
    }
    /// Render with SDK windowing policy retained by the control.
    pub fn build(self, window: &mut Window, cx: &mut Context<M>) -> Stateful<Div> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        let parts = self.control.render_parts();
        parts.scroll.set_wheel_focus_owner(parts.binding.focus_handle().clone());
        let layout = self.flow.layout((0.0, 0.0));
        let (axis, extent) = match self.flow {
            ListBoxFlow::Horizontal { item_width, .. } => {
                (gpui_luma::controls::listbox::ListBoxAxis::Horizontal, item_width)
            }
            _ => (gpui_luma::controls::listbox::ListBoxAxis::Vertical, layout.item_height),
        };
        let content_sized = matches!(self.flow, ListBoxFlow::VerticalContent { .. });
        let virtual_window = if content_sized {
            let (estimate, buffer) = match parts.virtualization {
                ListBoxVirtualization::Measured { estimated_height, overscan } => (estimated_height, Some(overscan)),
                _ => (48.0, None),
            };
            Some(parts.scroll.content_window(parts.state, estimate, layout.spacing, buffer))
        } else {
            match parts.virtualization {
                ListBoxVirtualization::Uniform { overscan } => {
                    parts.scroll.virtual_window(parts.state, axis, extent, layout.spacing, overscan)
                }
                _ => None,
            }
        };
        let range = virtual_window.as_ref().map_or(0..parts.state.visible_items().len(), |window| window.range.clone());
        let horizontal = axis == gpui_luma::controls::listbox::ListBoxAxis::Horizontal;
        let spacer = |extent| {
            div()
                .flex_shrink_0()
                .when(horizontal, |s| s.w(px(extent)))
                .when(!horizontal, |s| s.h(px(extent)))
                .into_any_element()
        };
        let mut rows = Vec::new();
        if let Some(extent) = virtual_window.as_ref().and_then(|v| v.leading_space) {
            rows.push(spacer(extent));
        }
        for item in parts.state.visible_items().skip(range.start).take(range.len()) {
            let row = listbox_row(&look, (parts.item_id)(&item.key), item.state, parts.binding.focus_visible(window));
            let key = item.key.clone();
            let mut row = parts.binding.bind_row(row, item.key, item.state, cx, parts.on_input);
            if let Some(drag_drop) = &parts.drag_drop {
                row = drag_drop.bind_row(row, parts.state.drag_keys(&key), (parts.item_label)(item.item), cx);
            }
            let model = ListBoxItemRenderModel {
                item: item.item,
                selected: item.state.selected,
                active: item.state.active,
                enabled: item.state.enabled,
            };
            let selector = format!("{}-item-{}", self.id, item.visible_index);
            let mut rendered = row
                .debug_selector(move || selector.clone())
                .flex_shrink_0()
                .min_w_0()
                .when(horizontal, |row| row.w(px(extent)))
                .when(!horizontal, |row| row.w_full())
                .when(!content_sized, |row| row.h(px(layout.item_height)))
                .aria_label((parts.item_label)(item.item))
                .child((self.template)(&model, cx));
            if let Some(drag_drop) = &parts.drag_drop {
                rendered = rendered.relative();
                if cx.has_active_drag() {
                    for (edge, before) in [
                        (gpui_luma::infra::drag_drop::DropEdge::Before, Some(key)),
                        (
                            gpui_luma::infra::drag_drop::DropEdge::After,
                            parts.state.visible_items().nth(item.visible_index + 1).map(|item| item.key),
                        ),
                    ] {
                        let zone = gpui_luma::infra::drag_drop::DropZone {
                            axis: if horizontal {
                                gpui::Axis::Horizontal
                            } else {
                                gpui::Axis::Vertical
                            },
                            edge,
                            item_extent: px(if horizontal { extent } else { layout.item_height }),
                            following_gap: px(if before.is_some() { layout.spacing } else { 0.0 }),
                            marker_width: px(2.0),
                        };
                        let target = div().id(if edge == gpui_luma::infra::drag_drop::DropEdge::Before {
                            "insert-before"
                        } else {
                            "insert-after"
                        });
                        let target = if content_sized {
                            zone.apply_measured(target)
                        } else {
                            zone.apply(target)
                        };
                        let target = drag_drop.highlight(
                            target.border_color(gpui::transparent_black()),
                            look.resolve_role(SemanticRole::Focus).hsla(),
                        );
                        rendered = rendered.child(drag_drop.bind_target(target, before, cx));
                    }
                }
            }
            rows.push(rendered.into_any_element());
        }
        if let Some(extent) = virtual_window.as_ref().and_then(|v| v.trailing_space) {
            rows.push(spacer(extent));
        }
        let mut viewport = div()
            .id(format!("{}-viewport", self.id))
            .flex()
            .w_full()
            .min_w_0()
            .h(px(layout.viewport_height))
            .gap(px(layout.spacing))
            .when(horizontal, |v| v.overflow_x_scroll())
            .when(!horizontal, |v| v.flex_col().overflow_y_scroll())
            .children(rows);
        if let Some(drag_drop) = &parts.drag_drop {
            viewport = drag_drop.bind_viewport(viewport, parts.scroll, axis);
        }
        let mut surface = crate::tree_view_frame(&look).id(self.id).w_full().min_w_0().aria_label(self.id);
        if let Some(drag_drop) = &parts.drag_drop {
            use gpui_luma::infra::drag_drop::DragDropElementExt;
            surface = drag_drop.bind_target(surface, None, cx).cancel_drag_on_escape();
        }
        let surface = match virtual_window {
            Some(rendered) => parts.scroll.bind_virtualized(surface, viewport, parts.state, rendered),
            None => parts.scroll.bind(surface, viewport, parts.state),
        };
        parts.binding.bind_root(surface, axis, parts.state.selection_mode(), window, cx, parts.on_input)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod persistent_drag_tests {
    fn render_drag_list(
        side: usize,
        list: &mut gpui_luma::controls::listbox::ListBoxControl<DragHarness, u32, u32>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<DragHarness>,
    ) -> gpui::Stateful<gpui::Div> {
        super::ListBoxBuilder::new(
            if side == 0 { "sdk-left" } else { "sdk-right" },
            list,
            super::listbox_template(|row: &u32| row.to_string().into()),
        )
        .build(window, cx)
    }
    include!("../../sdk/tests/support/listbox_drag.rs");
}
