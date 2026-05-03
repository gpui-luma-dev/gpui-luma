use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, Entity, Hsla, IntoElement, MouseDownEvent, MouseUpEvent, Pixels,
    Render, SharedString, Subscription, Window, div, hsla, prelude::*, px,
};
use gpui_luma::controls::popup_selector::{
    ControlFocusState, PopupSelector, PopupSelectorEvent, PopupSelectorPlacement, PopupSelectorRenderModel,
    PopupSelectorTemplate, PopupSelectorTemplateHandlers, SelectorItem,
};
use gpui_luma::theme::InteractionState;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{format_compact_hsla, gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct PopupSelectorPane {
    selector_smart: Entity<PopupSelector>,
    selector_below: Entity<PopupSelector>,
    selector_above: Entity<PopupSelector>,
    selector_overlay: Entity<PopupSelector>,
    selector_swatch: Entity<PopupSelector>,
    state_preview: Entity<PopupSelectorStatePreview>,
    selection: String,
}

impl PopupSelectorPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            selector_smart: PopupSelector::new("popup-selector-smart-example")
                .label("Select status")
                .items(selector_items())
                .placement(PopupSelectorPlacement::Smart)
                .template(theme.popup_selector_template())
                .spawn(cx),
            selector_below: PopupSelector::new("popup-selector-below-example")
                .label("Below selector")
                .items(selector_items())
                .placement(PopupSelectorPlacement::BelowStart)
                .template(theme.popup_selector_template())
                .spawn(cx),
            selector_above: PopupSelector::new("popup-selector-above-example")
                .label("Above selector")
                .items(selector_items())
                .placement(PopupSelectorPlacement::AboveStart)
                .template(theme.popup_selector_template())
                .spawn(cx),
            selector_overlay: PopupSelector::new("popup-selector-overlay-example")
                .label("Overlay selector")
                .items(selector_items())
                .placement(PopupSelectorPlacement::OverlayOnTrigger)
                .template(theme.popup_selector_template())
                .spawn(cx),
            selector_swatch: PopupSelector::new("popup-selector-swatch-example")
                .label("Choose color")
                .items(swatch_items())
                .selected_id("emerald-500")
                .with_item_template(|item, _cx| {
                    let swatch = swatch_color(item.item.id().as_ref());
                    let selected_weight = if item.selected {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(12.0))
                                .rounded(px(2.0))
                                .bg(swatch)
                                .border_1()
                                .border_color(hsla(0.0, 0.0, 1.0, 0.18)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .child(div().font_weight(selected_weight).child(item.item.label_text().clone()))
                                .child(
                                    div()
                                        .font_family("Monaco")
                                        .text_size(px(10.0))
                                        .line_height(px(14.0))
                                        .opacity(0.72)
                                        .child(format_compact_hsla(swatch)),
                                ),
                        )
                })
                .template(theme.popup_selector_template())
                .spawn(cx),
            state_preview: cx.new(|_| PopupSelectorStatePreview::new(theme)),
            selection: "none".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.selector_smart, |app, _, event: &PopupSelectorEvent, cx| {
            app.panes.popup_selector.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.selector_below, |app, _, event: &PopupSelectorEvent, cx| {
            app.panes.popup_selector.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.selector_above, |app, _, event: &PopupSelectorEvent, cx| {
            app.panes.popup_selector.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.selector_overlay, |app, _, event: &PopupSelectorEvent, cx| {
            app.panes.popup_selector.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.selector_swatch, |app, _, event: &PopupSelectorEvent, cx| {
            app.panes.popup_selector.handle_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage(
            "Popup Selector",
            "Popup Selector",
            div()
                .w_full()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(self.selector_below.clone())
                                .child(self.selector_above.clone()),
                        )
                        .child(div().flex().items_center().gap_3().child(self.selector_overlay.clone()))
                        .child(self.selector_swatch.clone())
                        .child(div().text_color(chrome.body_text).child(format!("Selected: {}", self.selection)))
                        .child(self.state_preview.clone()),
                )
                .child(self.selector_smart.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.selector_smart, cx);
        notify_entity(&self.selector_below, cx);
        notify_entity(&self.selector_above, cx);
        notify_entity(&self.selector_overlay, cx);
        notify_entity(&self.selector_swatch, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &PopupSelectorEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PopupSelectorEvent::Change { label, .. } => {
                self.selection = label.to_string();
                cx.notify();
            }
        }
    }
}

#[derive(Clone)]
struct PopupSelectorStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn PopupSelectorTemplate>,
}

struct PopupSelectorStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
    focus: ControlFocusState,
}

impl PopupSelectorStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.popup_selector_template() }
    }
}

impl Render for PopupSelectorStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            PopupSelectorStateSample {
                id: "default",
                label: "Standard",
                state: InteractionState::default(),
                focus: ControlFocusState::default(),
            },
            PopupSelectorStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
            },
            PopupSelectorStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            PopupSelectorStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            PopupSelectorStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
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
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_trigger_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_trigger_sample(
    template: &Arc<dyn PopupSelectorTemplate>,
    sample: PopupSelectorStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("popup-selector-preview-trigger-{}", sample.id));
    let label = SharedString::from("Selector");
    let items = selector_items().into_iter().collect::<Vec<_>>();
    let model = PopupSelectorRenderModel {
        id: &id,
        label: &label,
        selected_icon: None,
        selected_index: None,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: PopupSelectorPlacement::BelowStart,
        active_path: None,
        enabled: !sample.state.disabled,
        item_template: None,
        focus: sample.focus,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, popup_selector_preview_handlers(items.len()), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn popup_selector_preview_handlers(root_count: usize) -> PopupSelectorTemplateHandlers {
    PopupSelectorTemplateHandlers {
        trigger_bounds: Box::new(noop_bounds),
        trigger_click: Box::new(noop_click),
        trigger_hover: Box::new(noop_hover),
        trigger_mouse_down: Box::new(noop_mouse_down),
        trigger_mouse_up: Box::new(noop_mouse_up),
        trigger_mouse_up_out: Box::new(noop_mouse_up),
        root_mouse_down_out: Box::new(noop_mouse_down),
        item_hovers: (0..root_count).map(|_| Box::new(noop_hover) as _).collect(),
        item_clicks: (0..root_count).map(|_| Box::new(noop_click) as _).collect(),
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn selector_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

fn swatch_items() -> [SelectorItem; 6] {
    [
        SelectorItem::new("sky-500").label("Sky 500 (#0EA5E9)"),
        SelectorItem::new("emerald-500").label("Emerald 500 (#10B981)"),
        SelectorItem::new("amber-500").label("Amber 500 (#F59E0B)"),
        SelectorItem::new("rose-500").label("Rose 500 (#F43F5E)"),
        SelectorItem::new("violet-500").label("Violet 500 (#8B5CF6)"),
        SelectorItem::new("slate-500").label("Slate 500 (#64748B)"),
    ]
}

fn swatch_color(id: &str) -> Hsla {
    match id {
        "sky-500" => hsla(0.55, 0.85, 0.48, 1.0),
        "emerald-500" => hsla(0.44, 0.85, 0.39, 1.0),
        "amber-500" => hsla(0.11, 0.92, 0.51, 1.0),
        "rose-500" => hsla(0.96, 0.89, 0.60, 1.0),
        "violet-500" => hsla(0.74, 0.84, 0.66, 1.0),
        "slate-500" => hsla(0.60, 0.18, 0.47, 1.0),
        _ => hsla(0.0, 0.0, 0.5, 1.0),
    }
}
