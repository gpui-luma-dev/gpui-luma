use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::radio_group::{
    ControlFocusState, RadioGroup, RadioGroupClickHandler, RadioGroupEvent, RadioGroupHoverHandler, RadioGroupItem,
    RadioGroupItemState, RadioGroupMouseDownHandler, RadioGroupMouseUpHandler, RadioGroupRenderItem,
    RadioGroupRenderModel, RadioGroupTemplate, RadioGroupTemplateHandlers,
};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct RadioGroupPane {
    radio_group: Entity<RadioGroup>,
    state_preview: Entity<RadioGroupStatePreview>,
    choice: String,
}

impl RadioGroupPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            radio_group: RadioGroup::new("density-radio-group")
                .items(density_items())
                .selected("comfortable")
                .template(theme.radio_group_template())
                .spawn(cx),
            state_preview: cx.new(|_| RadioGroupStatePreview::new(theme)),
            choice: "Comfortable".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.radio_group, |app, _, event: &RadioGroupEvent, cx| {
            app.panes.radio_group.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Radio Group",
            "Radio Group",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_5()
                .child(self.radio_group.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Choice: {}", self.choice)),
                )
                .child(self.state_preview.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.radio_group, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            RadioGroupEvent::Change { label, .. } => {
                self.choice = label.to_string();
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct RadioGroupStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn RadioGroupTemplate>,
}

#[derive(Clone, Copy)]
struct RadioGroupStateSample {
    id: &'static str,
    label: &'static str,
    state: RadioGroupItemState,
}

impl RadioGroupStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.radio_group_template() }
    }
}

impl Render for RadioGroupStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            RadioGroupStateSample { id: "default", label: "Default", state: RadioGroupItemState::default() },
            RadioGroupStateSample {
                id: "hover",
                label: "Hover",
                state: RadioGroupItemState { hovered: true, ..RadioGroupItemState::default() },
            },
            RadioGroupStateSample {
                id: "focus",
                label: "Focus",
                state: RadioGroupItemState { active: true, focus_visible: true, ..RadioGroupItemState::default() },
            },
            RadioGroupStateSample {
                id: "active",
                label: "Active",
                state: RadioGroupItemState {
                    hovered: true,
                    pressed: true,
                    active: true,
                    focus_visible: true,
                    ..RadioGroupItemState::default()
                },
            },
            RadioGroupStateSample {
                id: "disabled",
                label: "Disabled",
                state: RadioGroupItemState { disabled: true, ..RadioGroupItemState::default() },
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
    template: &Arc<dyn RadioGroupTemplate>,
    row_label: &'static str,
    selected: bool,
    samples: &[RadioGroupStateSample],
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
    template: &Arc<dyn RadioGroupTemplate>,
    selected: bool,
    sample: &RadioGroupStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("radio-group-preview-{}-{}", selected, sample.id));
    let item_ids = [SharedString::from("compact"), SharedString::from("comfortable"), SharedString::from("expanded")];
    let item_labels =
        [SharedString::from("Compact"), SharedString::from("Comfortable"), SharedString::from("Expanded")];
    let enabled = !sample.state.disabled;
    let selected_id = selected.then_some(&item_ids[1]);
    let focus = ControlFocusState { focused: sample.state.active, focus_visible: sample.state.focus_visible };
    let items = item_ids
        .iter()
        .zip(item_labels.iter())
        .enumerate()
        .map(|(index, (item_id, label))| {
            let target = index == 1;
            let item_selected = target && selected;
            let state = if target {
                RadioGroupItemState { selected: item_selected, ..sample.state }
            } else {
                RadioGroupItemState { disabled: !enabled, ..RadioGroupItemState::default() }
            };

            RadioGroupRenderItem { id: item_id, label, selected: item_selected, enabled, state }
        })
        .collect();
    let model = RadioGroupRenderModel { id: &id, items, selected_id, enabled, focus };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, radio_group_preview_handlers(3), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn radio_group_preview_handlers(count: usize) -> RadioGroupTemplateHandlers {
    RadioGroupTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as RadioGroupHoverHandler).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as RadioGroupMouseDownHandler).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as RadioGroupMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as RadioGroupMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as RadioGroupClickHandler).collect(),
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn density_items() -> [RadioGroupItem; 3] {
    [
        RadioGroupItem::new("compact").label("Compact"),
        RadioGroupItem::new("comfortable").label("Comfortable"),
        RadioGroupItem::new("expanded").label("Expanded"),
    ]
}
