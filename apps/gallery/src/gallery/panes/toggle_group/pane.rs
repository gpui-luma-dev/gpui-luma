use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::toggle_group::{
    ControlFocusState, ToggleGroup, ToggleGroupClickHandler, ToggleGroupEvent, ToggleGroupHoverHandler,
    ToggleGroupItem, ToggleGroupItemPosition, ToggleGroupItemState, ToggleGroupKind, ToggleGroupMouseDownHandler,
    ToggleGroupMouseUpHandler, ToggleGroupRenderItem, ToggleGroupRenderModel, ToggleGroupSelectionMode,
    ToggleGroupSize, ToggleGroupTemplate, ToggleGroupTemplateHandlers,
};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    single_group: Entity<ToggleGroup>,
    multiple_group: Entity<ToggleGroup>,
    state_preview: Entity<ToggleGroupStatePreview>,
    placement: String,
    visible_edges: Vec<String>,
}

impl ToggleGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            single_group: ToggleGroup::new("placement-toggle-group")
                .items(placement_items())
                .selected("bottom")
                .template(theme.toggle_group_template())
                .spawn(cx),
            multiple_group: ToggleGroup::new("edge-toggle-group")
                .multiple()
                .items(edge_items())
                .selected_ids(["top", "left"])
                .template(theme.toggle_group_template())
                .spawn(cx),
            state_preview: cx.new(|_| ToggleGroupStatePreview::new(theme)),
            placement: "Bottom".to_string(),
            visible_edges: vec!["Top".to_string(), "Left".to_string()],
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single_group, |app, _, event: &ToggleGroupEvent, cx| {
            app.panes.toggle_group.handle_single_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.multiple_group, |app, _, event: &ToggleGroupEvent, cx| {
            app.panes.toggle_group.handle_multiple_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Toggle Group",
            "Toggle Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(self.single_group.clone())
                        .child(div().text_color(chrome.body_text).child(format!("Single: {}", self.placement))),
                )
                .child(div().flex().flex_col().items_center().gap_2().child(self.multiple_group.clone()).child(
                    div().text_color(chrome.body_text).child(format!("Multiple: {}", self.visible_edges.join(", "))),
                ))
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.single_group, cx);
        notify_entity(&self.multiple_group, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_single_event(&mut self, event: &ToggleGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleGroupEvent::Change { label, selected, .. } => {
                self.placement = if *selected {
                    label.to_string()
                } else {
                    "None".to_string()
                };
                cx.notify();
            }
        }
    }

    fn handle_multiple_event(&mut self, event: &ToggleGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ToggleGroupEvent::Change { selected_ids, .. } => {
                self.visible_edges = selected_ids.iter().map(ToString::to_string).map(label_for_edge_id).collect();
            }
        }

        if self.visible_edges.is_empty() {
            self.visible_edges.push("None".to_string());
        }

        cx.notify();
    }
}

#[derive(Clone)]
struct ToggleGroupStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ToggleGroupTemplate>,
}

#[derive(Clone, Copy)]
struct ToggleGroupStateSample {
    id: &'static str,
    label: &'static str,
    state: ToggleGroupItemState,
}

impl ToggleGroupStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.toggle_group_template() }
    }
}

impl Render for ToggleGroupStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ToggleGroupStateSample { id: "default", label: "Default", state: ToggleGroupItemState::default() },
            ToggleGroupStateSample {
                id: "hover",
                label: "Hover",
                state: ToggleGroupItemState { hovered: true, ..ToggleGroupItemState::default() },
            },
            ToggleGroupStateSample {
                id: "focus",
                label: "Focus",
                state: ToggleGroupItemState { active: true, focus_visible: true, ..ToggleGroupItemState::default() },
            },
            ToggleGroupStateSample {
                id: "active",
                label: "Active",
                state: ToggleGroupItemState {
                    hovered: true,
                    pressed: true,
                    active: true,
                    focus_visible: true,
                    ..ToggleGroupItemState::default()
                },
            },
            ToggleGroupStateSample {
                id: "disabled",
                label: "Disabled",
                state: ToggleGroupItemState { disabled: true, ..ToggleGroupItemState::default() },
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(render_state_row(&self.template, "Unselected", false, &samples, chrome.muted_text, window, cx))
            .child(render_state_row(&self.template, "Selected", true, &samples, chrome.muted_text, window, cx))
    }
}

fn render_state_row(
    template: &Arc<dyn ToggleGroupTemplate>,
    row_label: &'static str,
    selected: bool,
    samples: &[ToggleGroupStateSample],
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(
            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples
                    .iter()
                    .map(|sample| render_state_sample(template, selected, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ToggleGroupTemplate>,
    selected: bool,
    sample: &ToggleGroupStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("toggle-group-preview-{}-{}", selected, sample.id));
    let item_ids = [SharedString::from("top"), SharedString::from("bottom"), SharedString::from("right")];
    let item_labels = [SharedString::from("Top"), SharedString::from("Bottom"), SharedString::from("Right")];
    let selected_ids = if selected {
        vec![item_ids[1].clone()]
    } else {
        Vec::new()
    };
    let active_id = (sample.state.active || sample.state.focus_visible).then_some(&item_ids[1]);
    let enabled = !sample.state.disabled;
    let focus = ControlFocusState { focused: sample.state.active, focus_visible: sample.state.focus_visible };
    let items = item_ids
        .iter()
        .zip(item_labels.iter())
        .enumerate()
        .map(|(index, (item_id, label))| {
            let target = index == 1;
            let item_selected = target && selected;
            let state = if target {
                ToggleGroupItemState { selected: item_selected, ..sample.state }
            } else {
                ToggleGroupItemState { disabled: !enabled, ..ToggleGroupItemState::default() }
            };

            ToggleGroupRenderItem {
                id: item_id,
                label,
                selected: item_selected,
                enabled,
                position: item_position(index, item_ids.len()),
                state,
            }
        })
        .collect();
    let model = ToggleGroupRenderModel {
        id: &id,
        items,
        selected_ids: &selected_ids,
        active_id,
        selection_mode: ToggleGroupSelectionMode::Single,
        kind: ToggleGroupKind::Default,
        size: ToggleGroupSize::Md,
        enabled,
        focus,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, toggle_group_preview_handlers(3), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn toggle_group_preview_handlers(count: usize) -> ToggleGroupTemplateHandlers {
    ToggleGroupTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as ToggleGroupHoverHandler).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as ToggleGroupMouseDownHandler).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as ToggleGroupMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as ToggleGroupMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as ToggleGroupClickHandler).collect(),
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn item_position(index: usize, item_count: usize) -> ToggleGroupItemPosition {
    match (index, item_count) {
        (_, 0 | 1) => ToggleGroupItemPosition::Only,
        (0, _) => ToggleGroupItemPosition::First,
        (index, item_count) if index + 1 == item_count => ToggleGroupItemPosition::Last,
        _ => ToggleGroupItemPosition::Middle,
    }
}

fn placement_items() -> [ToggleGroupItem; 4] {
    [
        ToggleGroupItem::new("top").label("Top"),
        ToggleGroupItem::new("bottom").label("Bottom"),
        ToggleGroupItem::new("left").label("Left"),
        ToggleGroupItem::new("right").label("Right"),
    ]
}

fn edge_items() -> [ToggleGroupItem; 4] {
    [
        ToggleGroupItem::new("top").label("Top"),
        ToggleGroupItem::new("bottom").label("Bottom"),
        ToggleGroupItem::new("left").label("Left"),
        ToggleGroupItem::new("right").label("Right"),
    ]
}

fn label_for_edge_id(id: String) -> String {
    match id.as_str() {
        "top" => "Top",
        "bottom" => "Bottom",
        "left" => "Left",
        "right" => "Right",
        _ => id.as_str(),
    }
    .to_string()
}
