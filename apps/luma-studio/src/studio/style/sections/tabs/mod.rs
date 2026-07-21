use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::tabs_navigation::{
    ControlFocusState as TabsControlFocusState, TabsNavigationClickHandler, TabsNavigationHoverHandler,
    TabsNavigationItem, TabsNavigationItemState, TabsNavigationMouseDownHandler, TabsNavigationMouseUpHandler,
    TabsNavigationRenderItem, TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers,
    TabsNavigationWidthMode,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::preview_handlers::{
    input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::shell::section_shell_with_width;

#[derive(Clone, Copy)]
struct TabsNavigationStateSample {
    id: &'static str,
    label: &'static str,
    active_index: usize,
    target_index: usize,
    target_state: TabsNavigationItemState,
    enabled: bool,
}

pub(crate) fn render_tabs_navigation_template_section(
    look: Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template = look.tabs_navigation_template();
    let samples = tabs_navigation_state_samples();

    section_shell_with_width(
        960.0,
        "Tabs",
        "Inactive, active, hover, focus, pressed, and disabled.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_tabs_navigation_state_sample(&template, sample, chrome.muted_text, window, cx)
                }),
            ))
            .into_any_element(),
    )
}

fn render_tabs_navigation_state_sample(
    template: &Arc<dyn TabsNavigationTemplate>,
    sample: TabsNavigationStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("luma-studio-tabs-navigation-preview-{}", sample.id));
    let items = tabs_navigation_preview_tabs();
    let active_id = items.get(sample.active_index).map(TabsNavigationItem::id);
    let render_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_enabled = sample.enabled && item.is_enabled();
            let active = active_id.is_some_and(|active_id| active_id == item.id());
            let mut state = TabsNavigationItemState { selected: active, ..TabsNavigationItemState::default() };

            if index == sample.target_index {
                state = sample.target_state;
                state.selected = active;
            }

            if !item_enabled {
                state.disabled = true;
                state.hovered = false;
                state.pressed = false;
                state.active = false;
                state.focus_visible = false;
            }

            TabsNavigationRenderItem { id: item.id(), label: item.label_text(), active, enabled: item_enabled, state }
        })
        .collect::<Vec<_>>();
    let model = TabsNavigationRenderModel {
        id: &id,
        size: ControlSize::Md,
        width_mode: TabsNavigationWidthMode::Intrinsic,
        items: render_items,
        active_id,
        enabled: sample.enabled,
        focus: TabsControlFocusState {
            focused: sample.target_state.active,
            focus_visible: sample.target_state.focus_visible,
        },
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, tabs_navigation_preview_handlers(items.len()), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn tabs_navigation_state_samples() -> [TabsNavigationStateSample; 7] {
    [
        TabsNavigationStateSample {
            id: "inactive",
            label: "Inactive",
            active_index: 1,
            target_index: 0,
            target_state: TabsNavigationItemState::default(),
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "active",
            label: "Active",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState { selected: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "hover",
            label: "Hover",
            active_index: 1,
            target_index: 0,
            target_state: TabsNavigationItemState { hovered: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "focus",
            label: "Focus",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                active: true,
                focus_visible: true,
                ..TabsNavigationItemState::default()
            },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "pressed",
            label: "Pressed",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                active: true,
                hovered: true,
                pressed: true,
                focus_visible: true,
                ..TabsNavigationItemState::default()
            },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "disabled-item",
            label: "Disabled item",
            active_index: 1,
            target_index: 2,
            target_state: TabsNavigationItemState { disabled: true, ..TabsNavigationItemState::default() },
            enabled: true,
        },
        TabsNavigationStateSample {
            id: "disabled-list",
            label: "Disabled list",
            active_index: 1,
            target_index: 1,
            target_state: TabsNavigationItemState {
                selected: true,
                disabled: true,
                ..TabsNavigationItemState::default()
            },
            enabled: false,
        },
    ]
}

fn tabs_navigation_preview_handlers(count: usize) -> TabsNavigationTemplateHandlers {
    TabsNavigationTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(input_noop_hover) as TabsNavigationHoverHandler).collect(),
        item_mouse_downs: (0..count)
            .map(|_| Box::new(input_noop_mouse_down) as TabsNavigationMouseDownHandler)
            .collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(input_noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(input_noop_click) as TabsNavigationClickHandler).collect(),
    }
}

fn tabs_navigation_preview_tabs() -> [TabsNavigationItem; 3] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}
