use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, hsla, prelude::*, px,
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

use super::super::shared::{gallery_pane_with_usage_description, notify_entity};

const CHOICE_GROUP_DESCRIPTION: &str = concat!(
    "Choice Group supports radio-like single-selection semantics. ",
    "This pane demonstrates ChoiceGroup with RadioButton template rendering."
);

#[derive(Clone)]
pub(in crate::gallery) struct ChoiceGroupPane {
    optional_group: ChoiceGroup,
    required_group: ChoiceGroup,
    state_preview: Entity<ChoiceGroupStatePreview>,
    choice: String,
    required_choice: String,
    required_selected_id: SharedString,
}

impl ChoiceGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let transparent = hsla(0.0, 0.0, 0.0, 0.0);

        let optional_group = choice_group::single_select("choice-density-group")
            .items(density_items())
            .bool_button_template(theme.radio_button_template())
            .with_modifier(move |el, _| el.bg(transparent).border_color(transparent))
            .spawn(cx);

        let transparent = hsla(0.0, 0.0, 0.0, 0.0);
        let required_group = choice_group::single_select("choice-required-density-group")
            .managed_selected("comfortable")
            .items(density_items())
            .bool_button_template(theme.radio_button_template())
            .with_modifier(move |el, _| el.bg(transparent).border_color(transparent))
            .spawn(cx);

        Self {
            optional_group,
            required_group,
            state_preview: cx.new(|_| ChoiceGroupStatePreview::new(theme)),
            choice: "None".to_string(),
            required_choice: "Comfortable".to_string(),
            required_selected_id: SharedString::from("comfortable"),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.optional_group, |app, _, event: &ChoiceGroupEvent, cx| {
            app.panes.choice_group.handle_optional_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.required_group, |app, _, event: &ChoiceGroupEvent, cx| {
            app.panes.choice_group.handle_required_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage_description(
            "Choice Group",
            Some(CHOICE_GROUP_DESCRIPTION),
            "Choice Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(render_live_example("Allow none", self.optional_group.clone(), &self.choice, chrome.body_text))
                .child(render_live_example(
                    "Required selection",
                    self.required_group.clone(),
                    &self.required_choice,
                    chrome.body_text,
                ))
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.optional_group, cx);
        notify_entity(&self.required_group, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_optional_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected, .. } => {
                self.choice = if *selected {
                    label.to_string()
                } else {
                    "None".to_string()
                };
                cx.notify();
            }
        }
    }

    fn handle_required_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { item_id, label, selected, selected_ids, .. } => {
                if *selected {
                    self.required_selected_id = item_id.clone();
                    self.required_choice = label.to_string();
                }

                let fallback = self.required_selected_id.clone();
                let commit = if selected_ids.is_empty() {
                    vec![fallback]
                } else {
                    selected_ids.clone()
                };

                self.required_group.update(cx, move |group, cx| group.set_managed_selected_ids(commit.clone(), cx));

                cx.notify();
            }
        }
    }
}

fn render_live_example(label: &'static str, group: ChoiceGroup, choice: &str, text_color: gpui::Hsla) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(label),
        )
        .child(group)
        .child(
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .text_color(text_color)
                .child(format!("Choice: {choice}")),
        )
        .into_any_element()
}

#[derive(Clone)]
struct ChoiceGroupStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn ChoiceGroupTemplate>,
    item_template: gpui_luma::controls::choice_group::ChoiceGroupItemButtonTemplate,
}

#[derive(Clone, Copy)]
struct ChoiceGroupStateSample {
    id: &'static str,
    label: &'static str,
    state: ChoiceGroupItemState,
}

impl ChoiceGroupStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self {
            theme: theme.clone(),
            template: default_choice_group_template(),
            item_template: theme.radio_button_template(),
        }
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
            .child(render_state_row(
                &self.template,
                &self.item_template,
                "Unselected",
                false,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                &self.item_template,
                "Selected",
                true,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_state_row(
    template: &Arc<dyn ChoiceGroupTemplate>,
    item_template: &gpui_luma::controls::choice_group::ChoiceGroupItemButtonTemplate,
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
                samples.iter().map(|sample| {
                    render_state_sample(template, item_template, selected, sample, label_color, window, cx)
                }),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ChoiceGroupTemplate>,
    item_template: &gpui_luma::controls::choice_group::ChoiceGroupItemButtonTemplate,
    selected: bool,
    sample: &ChoiceGroupStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("choice-group-preview-{}-{}", selected, sample.id));
    let item_ids = [SharedString::from("compact"), SharedString::from("comfortable"), SharedString::from("expanded")];
    let item_labels =
        [SharedString::from("Compact"), SharedString::from("Comfortable"), SharedString::from("Expanded")];
    let selected_ids = if selected {
        vec![item_ids[1].clone()]
    } else {
        Vec::new()
    };
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
        item_button_template: Some(item_template),
        selected_ids: &selected_ids,
        active_id: None,
        selection_mode: ChoiceGroupSelectionMode::Single,
        layout: ChoiceGroupLayout::Horizontal,
        state_mode: ChoiceGroupStateMode::Unmanaged,
        kind: ChoiceGroupKind::Standard,
        size: ChoiceGroupSize::Md,
        enabled,
        focus,
    };

    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let preview_template: Arc<dyn ChoiceGroupTemplate> = Arc::new(
        gpui_luma::controls::choice_group::ModifiedChoiceGroupTemplate::new(template.clone())
            .with_modifier(move |el, _| el.bg(transparent).border_color(transparent)),
    );

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(preview_template.render(&model, choice_group_preview_handlers(3), window, cx))
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
        (idx, count) if idx + 1 == count => ChoiceGroupItemPosition::Last,
        _ => ChoiceGroupItemPosition::Middle,
    }
}

fn density_items() -> [ChoiceGroupItem; 3] {
    [
        ChoiceGroupItem::new("compact", "compact").label("Compact"),
        ChoiceGroupItem::new("comfortable", "comfortable").label("Comfortable"),
        ChoiceGroupItem::new("expanded", "expanded").label("Expanded"),
    ]
}
