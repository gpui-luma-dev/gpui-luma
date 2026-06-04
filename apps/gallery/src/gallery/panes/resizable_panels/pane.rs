use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, Context, Entity, IntoElement, ParentElement, SharedString, Subscription, div, prelude::*, px,
    transparent_black,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizablePanelSpec, ResizablePanels, ResizablePanelsEvent, ResizablePanelsOrientation,
};
use gpui_luma::theme::RadixTheme;

use crate::gallery::control::GalleryApp;

use super::super::shared::{gallery_pane_with_usage_description_scrollable, notify_entity};

const DEMO_WIDTH: f32 = 540.0;
const DEMO_HEIGHT: f32 = 220.0;
const NESTED_OUTER_WIDTH: f32 = 540.0;
const NESTED_OUTER_HEIGHT: f32 = 220.0;
const CONTROLLED_MIN_LEFT_PX: f32 = DEMO_WIDTH * 0.2;
const CONTROLLED_MAX_LEFT_PX: f32 = DEMO_WIDTH * 0.7;
const CONTROLLED_MIN_RIGHT_PX: f32 = DEMO_WIDTH * 0.3;
const CONTROLLED_MAX_RIGHT_PX: f32 = DEMO_WIDTH * 0.8;
const PANEL_BG_TRANSPARENT: gpui::Hsla = transparent_black();

#[derive(Clone)]
pub(in crate::gallery) struct ResizablePanelsPane {
    horizontal: Entity<ResizablePanels>,
    vertical: Entity<ResizablePanels>,
    nested_outer: Entity<ResizablePanels>,
    nested_inner: Entity<ResizablePanels>,
    controlled: Entity<ResizablePanels>,
    controlled_panel_sizes: Rc<Cell<[f32; 2]>>,
    reset_controlled_button: Entity<Button<()>>,
    horizontal_sizes: SharedString,
    vertical_sizes: SharedString,
    nested_outer_sizes: SharedString,
    nested_inner_sizes: SharedString,
    controlled_sizes: SharedString,
    controlled_last_event: SharedString,
}

impl ResizablePanelsPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let demo_theme = radix_theme.clone();

        let horizontal = radix_theme
            .resizable_panels("resizable-panels-horizontal")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Sidebar", &theme)
                })
                    .weight(3.0)
                    .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Content", &theme)
                })
                    .weight(7.0)
                    .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let vertical = radix_theme
            .resizable_panels("resizable-panels-vertical")
            .orientation(ResizablePanelsOrientation::Vertical)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Header", &theme)
                })
                    .weight(3.0)
                    .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Content", &theme)
                })
                    .weight(7.0)
                    .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let nested_inner = radix_theme
            .resizable_panels("resizable-panels-nested-inner")
            .orientation(ResizablePanelsOrientation::Vertical)
            .size(px(NESTED_OUTER_WIDTH * 0.5), px(NESTED_OUTER_HEIGHT))
            .show_border(false)
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Two", &theme)
                })
                    .weight(2.0)
                    .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Three", &theme)
                })
                    .weight(3.0)
                    .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let nested_inner_entity = nested_inner.clone();
        let nested_outer = radix_theme
            .resizable_panels("resizable-panels-nested-outer")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(NESTED_OUTER_WIDTH), px(NESTED_OUTER_HEIGHT))
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("One", &theme)
                })
                    .weight(1.0)
                    .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render(move || nested_inner_entity.clone())
                    .weight(1.0)
                    .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let controlled_panel_sizes = Rc::new(Cell::new([30.0_f32, 70.0_f32]));
        let left_sizes = controlled_panel_sizes.clone();
        let right_sizes = controlled_panel_sizes.clone();
        let controlled = radix_theme
            .resizable_panels("resizable-panels-controlled")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Lg)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || {
                        let sizes = left_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", sizes[0]))
                            .into_any_element()
                    }
                })
                .weight(3.0)
                .min(px(CONTROLLED_MIN_LEFT_PX))
                .max(px(CONTROLLED_MAX_LEFT_PX))
                .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || {
                        let sizes = right_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", sizes[1]))
                            .into_any_element()
                    }
                })
                .weight(7.0)
                .min(px(CONTROLLED_MIN_RIGHT_PX))
                .max(px(CONTROLLED_MAX_RIGHT_PX))
                .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let reset_controlled_button =
            Button::new("resizable-panels-controlled-reset").label("Reset Controlled (30 / 70)").spawn(cx);

        let mut this = Self {
            horizontal,
            vertical,
            nested_outer,
            nested_inner,
            controlled,
            controlled_panel_sizes,
            reset_controlled_button,
            horizontal_sizes: "30% / 70%".into(),
            vertical_sizes: "30% / 70%".into(),
            nested_outer_sizes: "50% / 50%".into(),
            nested_inner_sizes: "40% / 60%".into(),
            controlled_sizes: "30% / 70%".into(),
            controlled_last_event: "None".into(),
        };
        this.sync_nested_inner_size(cx);
        this
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.reset_controlled_button, |app, _, _: &ButtonEvent, cx| {
            app.panes.resizable_panels.reset_controlled(cx);
        }));
        subscriptions.push(cx.subscribe(&self.controlled, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes.resizable_panels.handle_controlled_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.nested_outer, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes.resizable_panels.handle_nested_outer_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.horizontal, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes.resizable_panels.apply_sizes_event(event, |pane, label| pane.horizontal_sizes = label, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes.resizable_panels.apply_sizes_event(event, |pane, label| pane.vertical_sizes = label, cx);
        }));
        subscriptions.push(cx.subscribe(&self.nested_inner, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes
                .resizable_panels
                .apply_sizes_event(event, |pane, label| pane.nested_inner_sizes = label, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        let chrome = radix_theme.chrome();

        gallery_pane_with_usage_description_scrollable(
            "Resizable Panels",
            Some(
                "Panel groups for horizontal, vertical, overlay handles (ResizeHandleSize), nested composition, and weight-based splits with pixel min/max.",
            ),
            "ResizablePanels",
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_6()
                .pb_4()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_4()
                        .child(frame("Horizontal", self.horizontal.clone(), chrome.border))
                        .child(frame("Vertical", self.vertical.clone(), chrome.border))
                        .child(frame("Nested", self.nested_outer.clone(), chrome.border))
                        .child(frame("Controlled", self.controlled.clone(), chrome.border)),
                )
                .child(
                    div().flex().items_center().gap_3().child(self.reset_controlled_button.clone()).child(
                        div()
                            .text_size(px(12.0))
                            .text_color(chrome.muted_text)
                            .child(format!("Last event: {}", self.controlled_last_event)),
                    ),
                )
                .child(telemetry_block(
                    chrome.muted_text,
                    chrome.title_text,
                    &self.horizontal_sizes,
                    &self.vertical_sizes,
                    &self.nested_outer_sizes,
                    &self.nested_inner_sizes,
                    &self.controlled_sizes,
                ))
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.horizontal, cx);
        notify_entity(&self.vertical, cx);
        notify_entity(&self.nested_outer, cx);
        notify_entity(&self.nested_inner, cx);
        notify_entity(&self.controlled, cx);
        notify_entity(&self.reset_controlled_button, cx);
    }

    fn reset_controlled(&mut self, cx: &mut Context<GalleryApp>) {
        self.controlled_panel_sizes.set([30.0, 70.0]);
        self.controlled.update(cx, |panels, cx| {
            panels.set_sizes(vec![30.0, 70.0], cx);
        });
        self.controlled_sizes = "30% / 70%".into();
        self.controlled_last_event = "Reset".into();
        cx.notify();
    }

    fn handle_controlled_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<GalleryApp>) {
        self.controlled_last_event = match event {
            ResizablePanelsEvent::ResizeStart => "ResizeStart".into(),
            ResizablePanelsEvent::SizesChanged { sizes } => {
                if sizes.len() >= 2 {
                    self.controlled_panel_sizes.set([sizes[0], sizes[1]]);
                }
                self.controlled_sizes = format_sizes(sizes).into();
                format!("SizesChanged: {}", format_sizes(sizes)).into()
            }
            ResizablePanelsEvent::ResizeEnd { sizes } => {
                if sizes.len() >= 2 {
                    self.controlled_panel_sizes.set([sizes[0], sizes[1]]);
                }
                self.controlled_sizes = format_sizes(sizes).into();
                format!("ResizeEnd: {}", format_sizes(sizes)).into()
            }
        };
        notify_entity(&self.controlled, cx);
        cx.notify();
    }

    fn handle_nested_outer_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ResizablePanelsEvent::SizesChanged { sizes } | ResizablePanelsEvent::ResizeEnd { sizes } => {
                self.nested_outer_sizes = format_sizes(sizes).into();
                self.sync_nested_inner_size(cx);
                cx.notify();
            }
            ResizablePanelsEvent::ResizeStart => {}
        }
    }

    fn apply_sizes_event(
        &mut self,
        event: &ResizablePanelsEvent,
        assign: impl FnOnce(&mut Self, SharedString),
        cx: &mut Context<GalleryApp>,
    ) {
        let sizes = match event {
            ResizablePanelsEvent::SizesChanged { sizes } | ResizablePanelsEvent::ResizeEnd { sizes } => sizes,
            ResizablePanelsEvent::ResizeStart => return,
        };
        assign(self, format_sizes(sizes).into());
        cx.notify();
    }

    fn sync_nested_inner_size(&mut self, cx: &mut Context<GalleryApp>) {
        let outer_sizes = self.nested_outer.read(cx).sizes();
        let right_percent = outer_sizes.get(1).copied().unwrap_or(50.0).clamp(0.0, 100.0) / 100.0;
        let content_width = NESTED_OUTER_WIDTH.max(1.0);
        let inner_width = (content_width * right_percent).max(1.0);
        self.nested_inner.update(cx, |nested, cx| {
            nested.set_frame_size(px(inner_width), px(NESTED_OUTER_HEIGHT), cx);
        });
    }
}

fn frame(title: &str, content: impl IntoElement, border_color: gpui::Hsla) -> impl IntoElement {
    let title = title.to_string();
    div().w(px(580.0)).flex().flex_col().gap_2().child(div().text_sm().child(title)).child(
        div()
            .w_full()
            .border_1()
            .border_dashed()
            .border_color(border_color)
            .rounded(px(10.0))
            .p_4()
            .child(content),
    )
}

fn telemetry_block(
    label_color: gpui::Hsla,
    title_color: gpui::Hsla,
    horizontal_sizes: &str,
    vertical_sizes: &str,
    nested_outer_sizes: &str,
    nested_inner_sizes: &str,
    controlled_sizes: &str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_sm().text_color(title_color).child("Telemetry"))
        .child(telemetry_row(label_color, "Horizontal sizes", horizontal_sizes))
        .child(telemetry_row(label_color, "Vertical sizes", vertical_sizes))
        .child(telemetry_row(label_color, "Nested outer sizes", nested_outer_sizes))
        .child(telemetry_row(label_color, "Nested inner sizes", nested_inner_sizes))
        .child(telemetry_row(label_color, "Controlled sizes", controlled_sizes))
}

fn telemetry_row(color: gpui::Hsla, label: &str, sizes: &str) -> impl IntoElement {
    div().text_size(px(12.0)).text_color(color).child(format!("{label}: {sizes}"))
}

fn format_sizes(sizes: &[f32]) -> String {
    sizes.iter().map(|size| format!("{size:.0}%")).collect::<Vec<_>>().join(" / ")
}

fn label_foreground(theme: &RadixTheme) -> gpui::Hsla {
    theme.token_color("foreground").unwrap_or_else(|_| theme.chrome().body_text)
}

fn demo_label(text: &str, theme: &RadixTheme) -> AnyElement {
    let text = text.to_string();
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(label_foreground(theme))
        .child(text)
        .into_any_element()
}
