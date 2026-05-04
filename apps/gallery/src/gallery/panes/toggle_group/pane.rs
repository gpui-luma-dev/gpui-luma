use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::choice_group::{
    self, ChoiceGroup, ChoiceGroupClickHandler, ChoiceGroupContent, ChoiceGroupEvent, ChoiceGroupHoverHandler,
    ChoiceGroupItem, ChoiceGroupItemContentModel, ChoiceGroupItemPosition, ChoiceGroupItemState, ChoiceGroupLayout,
    ChoiceGroupMouseDownHandler, ChoiceGroupMouseUpHandler, ChoiceGroupRenderItem, ChoiceGroupRenderModel,
    ChoiceGroupSelectionMode, ChoiceGroupStateMode, ChoiceGroupTemplate, ChoiceGroupTemplateHandlers, ChoiceGroupKind,
    ChoiceGroupSize, ControlFocusState, default_choice_group_template,
};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage_descriptions, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ToggleGroupPane {
    single_group: ChoiceGroup,
    multiple_group: ChoiceGroup,
    state_preview: Entity<ChoiceGroupStatePreview>,
    placement: String,
    visible_edges: Vec<String>,
}

impl ToggleGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            single_group: choice_group::new("placement-choice-group")
                .items(placement_items())
                .selected("bottom")
                .spawn(cx),
            multiple_group: choice_group::multiple("edge-choice-group")
                .items(edge_items())
                .selected_ids(["top", "left"])
                .spawn(cx),
            state_preview: cx.new(|_| ChoiceGroupStatePreview::new(theme)),
            placement: "Bottom".to_string(),
            visible_edges: vec!["Top".to_string(), "Left".to_string()],
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.single_group, |app, _, event: &ChoiceGroupEvent, cx| {
            app.panes.toggle_group.handle_single_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.multiple_group, |app, _, event: &ChoiceGroupEvent, cx| {
            app.panes.toggle_group.handle_multiple_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage_descriptions(
            "Toggle Group",
            None,
            &["Choice Group", "Toggle"],
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

    fn handle_single_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected, .. } => {
                self.placement = if *selected {
                    label.to_string()
                } else {
                    "None".to_string()
                };
                cx.notify();
            }
        }
    }

    fn handle_multiple_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { selected_ids, .. } => {
                self.visible_edges = selected_ids.iter().map(|id| label_for_edge_id(id.as_ref())).collect();
            }
        }

        if self.visible_edges.is_empty() {
            self.visible_edges.push("None".to_string());
        }

        cx.notify();
    }
}

#[derive(Clone)]
struct ChoiceGroupStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ChoiceGroupTemplate>,
}

#[derive(Clone, Copy)]
struct ChoiceGroupStateSample {
    id: &'static str,
    label: &'static str,
    state: ChoiceGroupItemState,
}

impl ChoiceGroupStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: default_choice_group_template() }
    }
}

impl Render for ChoiceGroupStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            ChoiceGroupStateSample { id: "default", label: "Standard", state: ChoiceGroupItemState::default() },
            ChoiceGroupStateSample {
                id: "hover",
                label: "Hover",
                state: ChoiceGroupItemState { hovered: true, ..ChoiceGroupItemState::default() },
            },
            ChoiceGroupStateSample {
                id: "focus",
                label: "Focus",
                state: ChoiceGroupItemState { active: true, focus_visible: true, ..ChoiceGroupItemState::default() },
            },
            ChoiceGroupStateSample {
                id: "active",
                label: "Active",
                state: ChoiceGroupItemState {
                    hovered: true,
                    pressed: true,
                    active: true,
                    focus_visible: true,
                    ..ChoiceGroupItemState::default()
                },
            },
            ChoiceGroupStateSample {
                id: "disabled",
                label: "Disabled",
                state: ChoiceGroupItemState { disabled: true, ..ChoiceGroupItemState::default() },
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
    template: &Arc<dyn ChoiceGroupTemplate>,
    row_label: &'static str,
    selected: bool,
    samples: &[ChoiceGroupStateSample],
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
    template: &Arc<dyn ChoiceGroupTemplate>,
    selected: bool,
    sample: &ChoiceGroupStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("choice-group-preview-{}-{}", selected, sample.id));
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
    let content: ChoiceGroupContent = Arc::new(|m, _| div().child(m.item_label.clone()).into_any_element());

    let items = item_ids
        .iter()
        .zip(item_labels.iter())
        .enumerate()
        .map(|(index, (item_id, label))| {
            let target = index == 1;
            let item_selected = target && selected;
            let state = if target {
                ChoiceGroupItemState { selected: item_selected, ..sample.state }
            } else {
                ChoiceGroupItemState { disabled: !enabled, ..ChoiceGroupItemState::default() }
            };

            let position = item_position(index, item_ids.len());
            let content_model = ChoiceGroupItemContentModel {
                group_id: id.clone(),
                item_id: item_id.clone(),
                item_label: label.clone(),
                item_value: item_id.clone(),
                selected: item_selected,
                enabled,
                position,
                state,
                selection_mode: ChoiceGroupSelectionMode::Single,
                layout: ChoiceGroupLayout::Horizontal,
                group_enabled: enabled,
            };

            ChoiceGroupRenderItem {
                id: item_id,
                label,
                value: item_id,
                selected: item_selected,
                enabled,
                position,
                state,
                content_model,
            }
        })
        .collect();

    let model = ChoiceGroupRenderModel {
        id: &id,
        items,
        content: &content,
        item_button_template: None,
        selected_ids: &selected_ids,
        active_id,
        selection_mode: ChoiceGroupSelectionMode::Single,
        layout: ChoiceGroupLayout::Horizontal,
        state_mode: ChoiceGroupStateMode::Unmanaged,
        kind: ChoiceGroupKind::Standard,
        size: ChoiceGroupSize::Md,
        enabled,
        focus,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, choice_group_preview_handlers(3), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn choice_group_preview_handlers(count: usize) -> ChoiceGroupTemplateHandlers {
    ChoiceGroupTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as ChoiceGroupHoverHandler).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as ChoiceGroupMouseDownHandler).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as ChoiceGroupMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as ChoiceGroupMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as ChoiceGroupClickHandler).collect(),
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn item_position(index: usize, item_count: usize) -> ChoiceGroupItemPosition {
    match (index, item_count) {
        (_, 0 | 1) => ChoiceGroupItemPosition::Only,
        (0, _) => ChoiceGroupItemPosition::First,
        (index, item_count) if index + 1 == item_count => ChoiceGroupItemPosition::Last,
        _ => ChoiceGroupItemPosition::Middle,
    }
}

fn placement_items() -> [ChoiceGroupItem; 4] {
    [
        ChoiceGroupItem::new("top", "top").label("Top"),
        ChoiceGroupItem::new("bottom", "bottom").label("Bottom"),
        ChoiceGroupItem::new("left", "left").label("Left"),
        ChoiceGroupItem::new("right", "right").label("Right"),
    ]
}

fn edge_items() -> [ChoiceGroupItem; 4] {
    [
        ChoiceGroupItem::new("top", "top").label("Top"),
        ChoiceGroupItem::new("bottom", "bottom").label("Bottom"),
        ChoiceGroupItem::new("left", "left").label("Left"),
        ChoiceGroupItem::new("right", "right").label("Right"),
    ]
}

fn label_for_edge_id(id: &str) -> String {
    match id {
        "top" => "Top",
        "bottom" => "Bottom",
        "left" => "Left",
        "right" => "Right",
        _ => id,
    }
    .to_string()
}
