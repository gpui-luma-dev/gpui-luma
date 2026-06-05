use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window,
    div, hsla, prelude::*, px,
};
use gpui_luma::controls::selection_panel::{
    SelectionPanelAppearance, SelectionPanelClickHandler, SelectionPanelControl, SelectionPanelEvent,
    SelectionPanelHoverHandler, SelectionPanelItem, SelectionPanelItemLike, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler, SelectionPanelRenderModel, SelectionPanelTemplate, item_template_with_modifier,
    make_selection_panel_item_template, template_with_modifier, default_selection_panel_template,
    render_selection_panel,
};
use gpui_luma::controls::state::ControlFocusState;
use gpui_luma_theme_radix::prelude::*;
use gpui_luma::theme::{ControlSize, ThemeMode};
use gpui_luma_theme_radix::RadixTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{format_compact_hsla, gallery_pane_with_usage_descriptions, notify_entity};
use super::layout::{PAGE_SPEC, render_live_panel_sample, render_page_header, render_section};

#[derive(Clone)]
pub(in crate::gallery) struct SelectionPanelPane {
    template_preview: Entity<SelectionPanelTemplatePreview>,
    interactive_panel: Entity<SelectionPanelControl<SelectionPanelItem>>,
    parameterized_panel: Entity<SelectionPanelControl<SelectionPanelItem>>,
    event_demo: Entity<SelectionPanelEventDemo>,
}

impl SelectionPanelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let interactive_items = (1..=20)
            .map(|index| {
                SelectionPanelItem::new(format!("item-{index}"))
                    .label(format!("Action item {index}"))
                    .icon(LucideIcon::ListChecks)
            })
            .collect::<Vec<_>>();

        let interactive_panel = radix_theme.selection_panel("gallery-selection-panel", cx);
        interactive_panel.update(cx, |panel, cx| {
            panel.set_panel_id("gallery-selection-panel-popup", cx);
            panel.set_items(interactive_items.clone(), cx);
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
                    let radix = radix_theme.clone();
                    move |size| {
                        let mut appearance = radix.selection_panel_appearance(size);
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

        let parameterized_panel = radix_theme.selection_panel("gallery-selection-panel-parameterized", cx);
        parameterized_panel.update(cx, |panel, cx| {
            panel.set_panel_id("gallery-selection-panel-parameterized-popup", cx);
            panel.set_items(interactive_items, cx);
            panel.with_template(
                make_parameterized_panel_template::<SelectionPanelItem>(ParameterizedPanelStyle {
                    open_opacity: 1.0,
                    closed_opacity: 0.88,
                }),
                cx,
            );
            panel.set_item_template(
                Some(make_parameterized_item_template(ParameterizedItemStyle {
                    active_badge_text: "ACTIVE",
                    active_badge_color: hsla(0.60, 0.70, 0.42, 1.0),
                    show_active_badge: true,
                })),
                cx,
            );
            panel.set_scrolling(true, cx);
            panel.set_appearance_provider(
                std::sync::Arc::new({
                    let radix = radix_theme.clone();
                    move |size| {
                        let mut appearance = radix.selection_panel_appearance(size);
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
            template_preview: cx.new(|_| SelectionPanelTemplatePreview::new(radix_theme.clone())),
            interactive_panel,
            parameterized_panel,
            event_demo: cx.new(|_| SelectionPanelEventDemo::new(radix_theme.clone())),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        let event_demo = self.event_demo.clone();
        subscriptions.push(cx.subscribe(&self.interactive_panel, move |_this, _, event: &SelectionPanelEvent, cx| {
            event_demo.update(cx, |demo, cx| {
                demo.record_event(event);
                cx.notify();
            });
        }));

        let event_demo = self.event_demo.clone();
        subscriptions.push(cx.subscribe(
            &self.parameterized_panel,
            move |_this, _, event: &SelectionPanelEvent, cx| {
                event_demo.update(cx, |demo, cx| {
                    demo.record_event(event);
                    cx.notify();
                });
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();
        let sections = PAGE_SPEC.sections;

        gallery_pane_with_usage_descriptions(
            PAGE_SPEC.title,
            Some(PAGE_SPEC.description),
            PAGE_SPEC.theme_components,
            div()
                .id("selection-panel-content")
                .h_full()
                .w_full()
                .min_h(px(0.0))
                .flex()
                .flex_col()
                .items_stretch()
                .justify_start()
                .gap(px(16.0))
                .overflow_y_scroll()
                .p(px(2.0))
                .child(render_page_header(PAGE_SPEC.header, radix_theme))
                .child(render_section(sections[0], self.template_preview.clone().into_any_element(), radix_theme))
                .child(render_section(
                    sections[1],
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .justify_center()
                        .gap(px(14.0))
                        .child(render_live_panel_sample(
                            "Baseline item template",
                            self.interactive_panel.clone().into_any_element(),
                            chrome.muted_text,
                            chrome.border,
                        ))
                        .child(render_live_panel_sample(
                            "Parameterized control + item templates",
                            self.parameterized_panel.clone().into_any_element(),
                            chrome.muted_text,
                            chrome.border,
                        ))
                        .into_any_element(),
                    radix_theme,
                ))
                .child(render_section(
                    sections[2],
                    div().flex().justify_center().child(self.event_demo.clone()).into_any_element(),
                    radix_theme,
                ))
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.template_preview, cx);
        notify_entity(&self.interactive_panel, cx);
        notify_entity(&self.parameterized_panel, cx);
        notify_entity(&self.event_demo, cx);
    }
}

#[derive(Clone)]
struct SelectionPanelTemplatePreview {
    radix_theme: Arc<RadixTheme>,
    template: Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
}

impl SelectionPanelTemplatePreview {
    fn new(radix_theme: Arc<RadixTheme>) -> Self {
        Self { radix_theme, template: default_selection_panel_template() }
    }
}

impl Render for SelectionPanelTemplatePreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.radix_theme.chrome();
        let mut appearance = self.radix_theme.selection_panel_appearance(ControlSize::Md);
        appearance.min_width = 220.0;
        appearance.padding = 7.0;

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
            .gap(px(14.0))
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

struct SelectionPanelEventDemo {
    radix_theme: Arc<RadixTheme>,
    hover_changes: usize,
    activate_rows: usize,
    active_index_changes: usize,
    last_event: SharedString,
}

impl SelectionPanelEventDemo {
    fn new(radix_theme: Arc<RadixTheme>) -> Self {
        Self {
            radix_theme,
            hover_changes: 0,
            activate_rows: 0,
            active_index_changes: 0,
            last_event: SharedString::default(),
        }
    }

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
        let style = event_demo_style(self.radix_theme.mode());

        div()
            .w(px(500.0))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .p(px(14.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(style.border)
            .bg(style.card_background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(10.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(style.title)
                            .child("SelectionPanelEvent demo"),
                    )
                    .child(
                        div()
                            .px(px(7.0))
                            .py(px(3.0))
                            .rounded(px(999.0))
                            .border_1()
                            .border_color(style.live_border)
                            .bg(style.live_background)
                            .text_size(px(10.0))
                            .line_height(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(style.live_text)
                            .child("live"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.0))
                    .child(render_event_count("HoverChanged", self.hover_changes, style))
                    .child(render_event_count("ActiveIndexChanged", self.active_index_changes, style))
                    .child(render_event_count("ActivateRow", self.activate_rows, style)),
            )
            .child(
                div()
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(style.code_border)
                    .bg(style.code_background)
                    .p(px(8.0))
                    .text_size(px(11.0))
                    .line_height(px(16.0))
                    .font_family("Monaco")
                    .text_color(style.body)
                    .child(if self.last_event.is_empty() {
                        SharedString::from("Last event: none")
                    } else {
                        SharedString::from(format!("Last event: {}", self.last_event))
                    }),
            )
    }
}

#[derive(Clone, Copy)]
struct EventDemoStyle {
    card_background: gpui::Hsla,
    border: gpui::Hsla,
    title: gpui::Hsla,
    body: gpui::Hsla,
    chip_background: gpui::Hsla,
    chip_border: gpui::Hsla,
    live_background: gpui::Hsla,
    live_border: gpui::Hsla,
    live_text: gpui::Hsla,
    code_background: gpui::Hsla,
    code_border: gpui::Hsla,
}

fn event_demo_style(mode: ThemeMode) -> EventDemoStyle {
    match mode {
        ThemeMode::Light => EventDemoStyle {
            card_background: hsla(0.61, 0.40, 0.15, 0.98),
            border: hsla(0.60, 0.32, 0.34, 0.55),
            title: hsla(0.33, 0.56, 0.70, 1.0),
            body: hsla(0.0, 0.0, 0.95, 0.96),
            chip_background: hsla(0.62, 0.28, 0.18, 0.74),
            chip_border: hsla(0.59, 0.26, 0.42, 0.66),
            live_background: hsla(0.62, 0.22, 0.20, 0.72),
            live_border: hsla(0.58, 0.30, 0.52, 0.62),
            live_text: hsla(0.0, 0.0, 0.93, 0.88),
            code_background: hsla(0.62, 0.24, 0.17, 0.82),
            code_border: hsla(0.58, 0.22, 0.44, 0.56),
        },
        ThemeMode::Dark => EventDemoStyle {
            card_background: hsla(0.62, 0.34, 0.14, 0.98),
            border: hsla(0.59, 0.26, 0.42, 0.62),
            title: hsla(0.34, 0.54, 0.68, 0.96),
            body: hsla(0.0, 0.0, 0.90, 0.94),
            chip_background: hsla(0.63, 0.24, 0.20, 0.78),
            chip_border: hsla(0.59, 0.24, 0.45, 0.68),
            live_background: hsla(0.63, 0.20, 0.22, 0.78),
            live_border: hsla(0.58, 0.26, 0.54, 0.66),
            live_text: hsla(0.0, 0.0, 0.90, 0.86),
            code_background: hsla(0.63, 0.20, 0.18, 0.86),
            code_border: hsla(0.58, 0.20, 0.48, 0.60),
        },
    }
}

fn render_event_count(label: &'static str, count: usize, style: EventDemoStyle) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.0))
        .rounded(px(5.0))
        .border_1()
        .border_color(style.chip_border)
        .bg(style.chip_background)
        .px(px(7.0))
        .py(px(5.0))
        .child(
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(style.body)
                .child(count.to_string()),
        )
        .child(div().text_size(px(10.0)).line_height(px(14.0)).text_color(style.body).opacity(0.76).child(label))
        .into_any_element()
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
                item_template: None,
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

#[derive(Clone, Copy)]
struct ParameterizedPanelStyle {
    open_opacity: f32,
    closed_opacity: f32,
}

#[derive(Clone, Copy)]
struct ParameterizedItemStyle {
    active_badge_text: &'static str,
    active_badge_color: gpui::Hsla,
    show_active_badge: bool,
}

fn make_parameterized_panel_template<T>(style: ParameterizedPanelStyle) -> Arc<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
{
    template_with_modifier(default_selection_panel_template(), move |root, model| {
        root.opacity(if model.open {
            style.open_opacity
        } else {
            style.closed_opacity
        })
    })
}

fn make_parameterized_item_template(
    style: ParameterizedItemStyle,
) -> gpui_luma::controls::selection_panel::SelectionPanelItemTemplate<SelectionPanelItem> {
    let base = make_selection_panel_item_template(
        |item: &gpui_luma::controls::selection_panel::SelectionPanelItemRenderModel<'_, SelectionPanelItem>, _cx| {
            let swatch = swatch_color(item.item.id().as_ref());
            let selected_weight = if item.selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            };

            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div().size(px(18.0)).rounded(px(2.0)).bg(swatch).border_1().border_color(hsla(0.0, 0.0, 1.0, 0.18)),
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
    );

    item_template_with_modifier(base, move |content, item, _cx| {
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .when(style.show_active_badge && item.active, |row| {
                row.child(
                    div()
                        .font_family("Monaco")
                        .text_size(px(9.0))
                        .line_height(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(style.active_badge_color)
                        .child(style.active_badge_text),
                )
            })
            .child(content)
            .into_any_element()
    })
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
