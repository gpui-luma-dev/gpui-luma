use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window,
    div, hsla, prelude::*, px,
};
use gpui_luma::controls::selection_panel::{
    SelectionPanelAppearance, SelectionPanelClickHandler, SelectionPanelControl, SelectionPanelEvent,
    SelectionPanelHoverHandler, SelectionPanelItem, SelectionPanelItemLike, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler, SelectionPanelRenderModel, SelectionPanelTemplate,
    default_selection_panel_appearance, default_selection_panel_template, render_selection_panel,
};
use gpui_luma::controls::state::ControlFocusState;
use gpui_luma::theme::ControlSize;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, gallery_pane_with_usage_descriptions, notify_entity};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone)]
pub(in crate::gallery) struct SelectionPanelPane {
    template_preview: Entity<SelectionPanelTemplatePreview>,
    interactive_panel: Entity<SelectionPanelControl<SelectionPanelItem>>,
    event_demo: Entity<SelectionPanelEventDemo>,
}

impl SelectionPanelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let interactive_items = (1..=20)
            .map(|index| {
                SelectionPanelItem::new(format!("item-{index}"))
                    .label(format!("Action item {index}"))
                    .icon(LucideIcon::ListChecks)
            })
            .collect::<Vec<_>>();

        let interactive_panel = theme.selection_panel("gallery-selection-panel", cx);

        interactive_panel.update(cx, |panel, cx| {
            panel.set_panel_id("gallery-selection-panel-popup", cx);
            panel.set_items(interactive_items, cx);
            panel.with_item_template(
                |item, _cx| {
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
                                .size(px(18.0))
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
                },
                cx,
            );
            panel.set_scrolling(true, cx);
            panel.set_appearance_provider(
                std::sync::Arc::new({
                    let interactive_theme = theme.clone();
                    move |size| {
                        let mut appearance = default_selection_panel_appearance(&interactive_theme.tokens(), size);
                        appearance.min_width = 320.0;
                        appearance
                    }
                }),
                cx,
            );
            panel.set_visible_row_limits(5, 5, cx);
            panel.set_selected_source_index(Some(1), cx);
            panel.set_active_visible_index(Some(1), cx);
        });

        Self {
            template_preview: cx.new(|_| SelectionPanelTemplatePreview::new(theme.clone())),
            interactive_panel,
            event_demo: cx.new(|_| SelectionPanelEventDemo::default()),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        let event_demo = self.event_demo.clone();
        subscriptions.push(cx.subscribe(&self.interactive_panel, move |_this, _, event: &SelectionPanelEvent, cx| {
            let _ = event_demo.update(cx, |demo, cx| {
                demo.record_event(event);
                cx.notify();
            });
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage_descriptions(
            "Selection Panel",
            Some("Template preview + spawnable SDK control with live SelectionPanelEvent stream."),
            &["Selection Panel"],
            div()
                .id("selection-panel-content")
                .h_full()
                .w_full()
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .items_center()
                .justify_start()
                .gap(px(14.0))
                .overflow_y_scroll()
                .child(self.template_preview.clone())
                .child(
                    div().flex().flex_col().items_center().gap(px(8.0)).child(self.interactive_panel.clone()).child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(15.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.muted_text)
                            .child("Hover, click, Arrow keys, Home/End, Enter to emit events"),
                    ),
                )
                .child(self.event_demo.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.template_preview, cx);
        notify_entity(&self.interactive_panel, cx);
        notify_entity(&self.event_demo, cx);
    }
}

#[derive(Clone)]
struct SelectionPanelTemplatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
}

impl SelectionPanelTemplatePreview {
    fn new(theme: GalleryThemePack) -> Self {
        Self { theme, template: default_selection_panel_template() }
    }
}

impl Render for SelectionPanelTemplatePreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let appearance = default_selection_panel_appearance(&self.theme.tokens(), ControlSize::Md);

        let standard_items = vec![
            SelectionPanelItem::new("draft-note").label("Draft note").icon(LucideIcon::FilePenLine),
            SelectionPanelItem::new("pin-sidebar").label("Pin in sidebar").icon(LucideIcon::Pin),
            SelectionPanelItem::new("mark-complete").label("Mark complete").icon(LucideIcon::CircleCheck),
        ];

        let active_items = vec![
            SelectionPanelItem::new("review").label("Review pending").icon(LucideIcon::ClipboardCheck),
            SelectionPanelItem::new("escalate").label("Escalate").icon(LucideIcon::TriangleAlert),
            SelectionPanelItem::new("defer").label("Defer 24h").icon(LucideIcon::Clock3),
        ];

        let disabled_items = vec![
            SelectionPanelItem::new("diagnostics").label("Open diagnostics").icon(LucideIcon::ScanSearch),
            SelectionPanelItem::new("sync").label("Sync records").icon(LucideIcon::RefreshCcw).enabled(false),
            SelectionPanelItem::new("export").label("Export bundle").icon(LucideIcon::PackageOpen),
        ];

        div()
            .flex()
            .flex_wrap()
            .items_start()
            .justify_center()
            .gap(px(16.0))
            .child(render_template_sample(
                "Standard",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-standard",
                &standard_items,
                None,
                None,
                cx,
            ))
            .child(render_template_sample(
                "Hover / active item",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-active",
                &active_items,
                Some(1),
                Some(1),
                cx,
            ))
            .child(render_template_sample(
                "Disabled item",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-disabled",
                &disabled_items,
                None,
                None,
                cx,
            ))
    }
}

#[derive(Default)]
struct SelectionPanelEventDemo {
    hover_changes: usize,
    activate_rows: usize,
    active_index_changes: usize,
    last_event: SharedString,
}

impl SelectionPanelEventDemo {
    fn record_event(&mut self, event: &SelectionPanelEvent) {
        match event {
            SelectionPanelEvent::HoverChanged { visible_index } => {
                self.hover_changes += 1;
                self.last_event = SharedString::from(format!("HoverChanged -> {:?}", visible_index));
            }
            SelectionPanelEvent::ActivateRow { source_index, visible_index, item_id } => {
                self.activate_rows += 1;
                self.last_event = SharedString::from(format!(
                    "ActivateRow -> source={}, visible={}, id={}",
                    source_index, visible_index, item_id
                ));
            }
            SelectionPanelEvent::ActiveIndexChanged { visible_index } => {
                self.active_index_changes += 1;
                self.last_event = SharedString::from(format!("ActiveIndexChanged -> {:?}", visible_index));
            }
        }
    }
}

impl Render for SelectionPanelEventDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(420.0))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .p(px(10.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(gpui::hsla(0.0, 0.0, 0.65, 0.6))
            .bg(gpui::hsla(0.0, 0.0, 1.0, 0.04))
            .child(
                div()
                    .text_size(px(11.0))
                    .line_height(px(15.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(gpui::hsla(0.0, 0.0, 0.2, 1.0))
                    .child("SelectionPanelEvent demo"),
            )
            .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(gpui::hsla(0.0, 0.0, 0.3, 1.0)).child(
                format!(
                    "HoverChanged: {} | ActiveIndexChanged: {} | ActivateRow: {}",
                    self.hover_changes, self.active_index_changes, self.activate_rows
                ),
            ))
            .child(
                div()
                    .text_size(px(11.0))
                    .line_height(px(15.0))
                    .font_family("Monaco")
                    .text_color(gpui::hsla(0.0, 0.0, 0.35, 1.0))
                    .child(if self.last_event.is_empty() {
                        SharedString::from("Last event: none")
                    } else {
                        SharedString::from(format!("Last event: {}", self.last_event))
                    }),
            )
    }
}

fn render_template_sample(
    label: &'static str,
    template: &Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
    appearance: &SelectionPanelAppearance,
    label_color: gpui::Hsla,
    sample_id: &'static str,
    items: &[SelectionPanelItem],
    selected_index: Option<usize>,
    active_index: Option<usize>,
    cx: &mut App,
) -> AnyElement {
    let panel_id = SharedString::from(sample_id);
    let control_id = SharedString::from(format!("{sample_id}-control"));
    let visible_indices = (0..items.len()).collect::<Vec<_>>();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(render_selection_panel(
            template.clone(),
            &SelectionPanelRenderModel {
                panel_id: &panel_id,
                control_id: &control_id,
                items,
                visible_indices: &visible_indices,
                selected_source_index: selected_index,
                active_visible_index: active_index,
                hovered_visible_index: active_index,
                pressed_visible_index: None,
                open: true,
                enabled: true,
                focus: ControlFocusState::default(),
                presenter: None,
                appearance: appearance.clone(),
                show_selection_marker: true,
                show_panel_chrome: true,
            },
            noop_hovers(items.len()),
            noop_mouse_downs(items.len()),
            noop_mouse_ups(items.len()),
            noop_mouse_ups(items.len()),
            noop_clicks(items.len()),
            cx,
        ))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}

fn noop_hovers(count: usize) -> Vec<SelectionPanelHoverHandler> {
    (0..count)
        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as SelectionPanelHoverHandler)
        .collect()
}

fn noop_mouse_downs(count: usize) -> Vec<SelectionPanelMouseDownHandler> {
    (0..count)
        .map(|_| Box::new(|_: &gpui::MouseDownEvent, _: &mut Window, _: &mut App| {}) as SelectionPanelMouseDownHandler)
        .collect()
}

fn noop_mouse_ups(count: usize) -> Vec<SelectionPanelMouseUpHandler> {
    (0..count)
        .map(|_| Box::new(|_: &gpui::MouseUpEvent, _: &mut Window, _: &mut App| {}) as SelectionPanelMouseUpHandler)
        .collect()
}

fn noop_clicks(count: usize) -> Vec<SelectionPanelClickHandler> {
    (0..count)
        .map(|_| Box::new(|_: &ClickEvent, _: &mut Window, _: &mut App| {}) as SelectionPanelClickHandler)
        .collect()
}

fn swatch_color(item_id: &str) -> gpui::Hsla {
    let swatches: [gpui::Hsla; 12] = [
        gpui::hsla(0.55, 0.85, 0.48, 1.0),
        gpui::hsla(0.44, 0.85, 0.39, 1.0),
        gpui::hsla(0.11, 0.92, 0.51, 1.0),
        gpui::hsla(0.96, 0.89, 0.60, 1.0),
        gpui::hsla(0.74, 0.84, 0.66, 1.0),
        gpui::hsla(0.60, 0.18, 0.47, 1.0),
        gpui::hsla(0.66, 0.84, 0.67, 1.0),
        gpui::hsla(0.52, 0.94, 0.43, 1.0),
        gpui::hsla(0.48, 0.81, 0.40, 1.0),
        gpui::hsla(0.23, 0.80, 0.44, 1.0),
        gpui::hsla(0.13, 0.93, 0.47, 1.0),
        gpui::hsla(0.07, 0.95, 0.53, 1.0),
    ];

    let hash = item_id.bytes().fold(0usize, |acc, byte| acc.wrapping_mul(31).wrapping_add(byte as usize));
    swatches[hash % swatches.len()]
}
