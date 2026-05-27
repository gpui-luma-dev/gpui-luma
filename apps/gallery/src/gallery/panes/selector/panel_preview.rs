use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px,
};
use gpui_luma::controls::selector::ControlFocusState;
use gpui_luma::controls::selector_panel::{
    SelectorItem, SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, SelectorPath, default_selector_items_template,
};
use gpui_luma::theme::{ControlSize, RadixTheme};

#[derive(Clone)]
pub(super) struct SelectorPanelPreview {
    radix_theme: Arc<RadixTheme>,
    template: Arc<dyn SelectorItemsTemplate<SelectorItem>>,
}

struct SelectorPanelSample {
    id: &'static str,
    label: &'static str,
    items: Vec<SelectorItem>,
    selected_index: Option<usize>,
    active_path: Option<SelectorPath>,
}

impl SelectorPanelPreview {
    pub(super) fn new(radix_theme: Arc<RadixTheme>) -> Self {
        Self { radix_theme, template: default_selector_items_template() }
    }
}

impl Render for SelectorPanelPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let appearance = self.radix_theme.selector_items_panel_appearance(ControlSize::Md);

        let samples = vec![
            SelectorPanelSample {
                id: "default",
                label: "Default",
                items: default_items(),
                selected_index: None,
                active_path: None,
            },
            SelectorPanelSample {
                id: "active",
                label: "Active item",
                items: default_items(),
                selected_index: None,
                active_path: Some(SelectorPath::Item(1)),
            },
            SelectorPanelSample {
                id: "selected",
                label: "Selected item",
                items: default_items(),
                selected_index: Some(2),
                active_path: None,
            },
            SelectorPanelSample {
                id: "disabled",
                label: "Disabled item",
                items: disabled_items(),
                selected_index: None,
                active_path: None,
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Selector Panel preview"),
            )
            .child(div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                samples.into_iter().map(|sample| {
                    render_sample(sample, self.template.clone(), appearance.clone(), chrome.muted_text, cx)
                }),
            ))
    }
}

fn render_sample(
    sample: SelectorPanelSample,
    template: Arc<dyn SelectorItemsTemplate<SelectorItem>>,
    appearance: gpui_luma::controls::selector_panel::SelectorItemsPanelAppearance,
    label_color: gpui::Hsla,
    cx: &mut App,
) -> AnyElement {
    let menu_id = SharedString::from(format!("selector-panel-preview-{}", sample.id));
    let selector_id = SharedString::from("selector-panel-preview-selector");
    let count = sample.items.len();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(
            &SelectorItemsRenderModel {
                menu_id: &menu_id,
                selector_id: &selector_id,
                items: &sample.items,
                selected_index: sample.selected_index,
                active_path: sample.active_path,
                open: true,
                enabled: true,
                focus: ControlFocusState::default(),
                item_template: None,
                appearance,
            },
            SelectorItemsTemplateHandlers { item_hovers: noop_hovers(count), item_clicks: noop_clicks(count) },
            cx,
        ))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn default_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("new").label("New"),
        SelectorItem::new("open").label("Open"),
        SelectorItem::new("archive").label("Archive"),
    ]
}

fn disabled_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("open").label("Open"),
        SelectorItem::new("download").label("Download").enabled(false),
        SelectorItem::new("share").label("Share"),
    ]
}

fn noop_hovers(count: usize) -> Vec<SelectorPanelHoverHandler> {
    (0..count).map(|_| Box::new(noop_hover) as SelectorPanelHoverHandler).collect()
}

fn noop_clicks(count: usize) -> Vec<SelectorPanelClickHandler> {
    (0..count).map(|_| Box::new(noop_click) as SelectorPanelClickHandler).collect()
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
