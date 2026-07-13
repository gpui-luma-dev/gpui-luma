use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, Context, Entity, IntoElement, ParentElement, SharedString, Subscription, div, prelude::*, px,
    transparent_black,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent, HasPresenter};
use gpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizeHandleVisibility, ResizablePanelSpec, ResizablePanels, ResizablePanelsEvent,
    ResizablePanelsOrientation,
};
use gpui_luma::theme::LumaTextStyle;
use gpui_luma::resizable_panels;
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_resizable_panels_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_scrollable_with_inspector_description, notify_entity, InspectorToggleRegistry};

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
    hover_handles: Entity<ResizablePanels>,
    nested_outer: Entity<ResizablePanels>,
    controlled: Entity<ResizablePanels>,
    controlled_panel_sizes: Rc<Cell<[f32; 2]>>,
    reset_controlled_button: Entity<Button<()>>,
    horizontal_sizes: SharedString,
    vertical_sizes: SharedString,
    hover_handles_sizes: SharedString,
    nested_outer_sizes: SharedString,
    controlled_sizes: SharedString,
    controlled_last_event: SharedString,
    inspector: Entity<ColorInspectorShell>,
}

impl ResizablePanelsPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "resizable-panels-inspector-tree",
            look.clone(),
            build_resizable_panels_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "resizable-panels-inspector",
                "resizable-panels-inspector-split",
                "resizable-panels-inspector-detail",
                build_resizable_panels_inspect_tree,
                cx,
            )
        });

        let demo_theme = look.clone();

        let horizontal = look
            .resizable_panels("resizable-panels-horizontal")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
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

        let vertical = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "resizable-panels-vertical",
            layout: Vertical,
            size: (px(DEMO_WIDTH), px(DEMO_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Header", &theme)
                } => weight(3.0), bg: PANEL_BG_TRANSPARENT;
                |
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Content", &theme)
                } => weight(7.0), bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let hover_handles = look
            .resizable_panels("resizable-panels-hover-handles")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .handle_visibility(ResizeHandleVisibility::Hover)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Primary", &theme)
                })
                .weight(1.0)
                .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Secondary", &theme)
                })
                .weight(1.0)
                .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let nested_outer = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "resizable-panels-nested-outer",
            layout: Horizontal,
            size: (px(NESTED_OUTER_WIDTH), px(NESTED_OUTER_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    move || demo_label("One", &theme)
                } => weight(1.0), bg: PANEL_BG_TRANSPARENT;
                |
                resizable_panels! {
                    cx,
                    theme = look.resizable_panels_theme(),
                    id: "resizable-panels-nested-inner",
                    layout: Vertical,
                    show_border: false,
                    height: px(NESTED_OUTER_HEIGHT),
                    panels: [
                        {
                            let theme = demo_theme.clone();
                            move || demo_label("Two", &theme)
                        } => weight(2.0), bg: PANEL_BG_TRANSPARENT;
                        |
                        {
                            let theme = demo_theme.clone();
                            move || demo_label("Three", &theme)
                        } => weight(3.0), bg: PANEL_BG_TRANSPARENT;
                    ]
                } => weight(1.0), bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let controlled_panel_sizes = Rc::new(Cell::new([30.0_f32, 70.0_f32]));
        let left_sizes = controlled_panel_sizes.clone();
        let right_sizes = controlled_panel_sizes.clone();
        let controlled = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "resizable-panels-controlled",
            layout: Horizontal,
            show_handle: true,
            resize_handle: Lg,
            handle_grip: true,
            size: (px(DEMO_WIDTH), px(DEMO_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    let left_sizes = left_sizes.clone();
                    move || {
                        let sizes = left_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", percent_of_content(sizes[0])))
                            .into_any_element()
                    }
                } => weight(3.0),
                    min: px(CONTROLLED_MIN_LEFT_PX),
                    max: px(CONTROLLED_MAX_LEFT_PX),
                    bg: PANEL_BG_TRANSPARENT;
                |
                {
                    let theme = demo_theme.clone();
                    let right_sizes = right_sizes.clone();
                    move || {
                        let sizes = right_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", percent_of_content(sizes[1])))
                            .into_any_element()
                    }
                } => weight(7.0),
                    min: px(CONTROLLED_MIN_RIGHT_PX),
                    max: px(CONTROLLED_MAX_RIGHT_PX),
                    bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let reset_controlled_button =
            Button::new("resizable-panels-controlled-reset").label("Reset Controlled (30 / 70)").spawn(cx);

        Self {
            horizontal,
            vertical,
            hover_handles,
            nested_outer,
            controlled,
            controlled_panel_sizes,
            reset_controlled_button,
            horizontal_sizes: "162px / 378px".into(),
            vertical_sizes: "66px / 154px".into(),
            hover_handles_sizes: "270px / 270px".into(),
            nested_outer_sizes: "270px / 270px".into(),
            controlled_sizes: "162px / 378px".into(),
            controlled_last_event: "None".into(),
            inspector,
        }
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
        subscriptions.push(cx.subscribe(&self.hover_handles, |app, _, event: &ResizablePanelsEvent, cx| {
            app.panes
                .resizable_panels
                .apply_sizes_event(event, |pane, label| pane.hover_handles_sizes = label, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_scrollable_with_inspector_description(
            "resizable-panels",
            "Resizable Panels",
            Some(
                "Panel groups for horizontal, vertical, overlay handles (ResizeHandleSize), nested composition, and weight-based splits with pixel min/max.",
            ),
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
                        .child(frame("Horizontal", self.horizontal.clone(), look, chrome.muted_text, chrome.border))
                        .child(frame("Vertical", self.vertical.clone(), look, chrome.muted_text, chrome.border))
                        .child(frame(
                            "Hover Handles",
                            self.hover_handles.clone(),
                            look,
                            chrome.muted_text,
                            chrome.border,
                        ))
                        .child(frame("Nested", self.nested_outer.clone(), look, chrome.muted_text, chrome.border))
                        .child(frame("Controlled", self.controlled.clone(), look, chrome.muted_text, chrome.border)),
                )
                .child(
                    div().flex().items_center().gap_3().child(self.reset_controlled_button.clone()).child(
                        div()
                            .typography_style(look.typography_scale(ShadcnTextSize::Xs))
                            .text_color(chrome.muted_text)
                            .child(format!("Last event: {}", self.controlled_last_event)),
                    ),
                )
                .child(telemetry_block(
                    look,
                    chrome.muted_text,
                    chrome.title_text,
                    [
                        ("Horizontal sizes", self.horizontal_sizes.as_ref()),
                        ("Vertical sizes", self.vertical_sizes.as_ref()),
                        ("Hover-handle sizes", self.hover_handles_sizes.as_ref()),
                        ("Nested sizes", self.nested_outer_sizes.as_ref()),
                        ("Controlled sizes", self.controlled_sizes.as_ref()),
                    ],
                ))
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.horizontal, cx);
        notify_entity(&self.vertical, cx);
        notify_entity(&self.hover_handles, cx);
        notify_entity(&self.nested_outer, cx);
        notify_entity(&self.controlled, cx);
        notify_entity(&self.reset_controlled_button, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn reset_controlled(&mut self, cx: &mut Context<GalleryApp>) {
        self.controlled_panel_sizes.set([162.0, 378.0]);
        self.controlled.update(cx, |panels, cx| {
            panels.set_weights(vec![3.0, 7.0], cx);
        });
        self.controlled_sizes = "162px / 378px".into();
        self.controlled_last_event = "Reset".into();
        cx.notify();
    }

    fn handle_controlled_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<GalleryApp>) {
        self.controlled_last_event = match event {
            ResizablePanelsEvent::ResizeStart => "ResizeStart".into(),
            ResizablePanelsEvent::SizesChanged { sizes_px } => {
                if sizes_px.len() >= 2 {
                    self.controlled_panel_sizes.set([sizes_px[0], sizes_px[1]]);
                }
                self.controlled_sizes = format_sizes_px(sizes_px).into();
                format!("SizesChanged: {}", format_sizes_px(sizes_px)).into()
            }
            ResizablePanelsEvent::ResizeEnd { sizes_px } => {
                if sizes_px.len() >= 2 {
                    self.controlled_panel_sizes.set([sizes_px[0], sizes_px[1]]);
                }
                self.controlled_sizes = format_sizes_px(sizes_px).into();
                format!("ResizeEnd: {}", format_sizes_px(sizes_px)).into()
            }
        };
        notify_entity(&self.controlled, cx);
        cx.notify();
    }

    fn handle_nested_outer_event(&mut self, event: &ResizablePanelsEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ResizablePanelsEvent::SizesChanged { sizes_px } | ResizablePanelsEvent::ResizeEnd { sizes_px } => {
                self.nested_outer_sizes = format_sizes_px(sizes_px).into();
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
            ResizablePanelsEvent::SizesChanged { sizes_px } | ResizablePanelsEvent::ResizeEnd { sizes_px } => sizes_px,
            ResizablePanelsEvent::ResizeStart => return,
        };
        assign(self, format_sizes_px(sizes).into());
        cx.notify();
    }
}

fn frame(
    title: &str,
    content: impl IntoElement,
    look: &ShadcnLook,
    label_color: gpui::Hsla,
    border_color: gpui::Hsla,
) -> impl IntoElement {
    let title = title.to_string();
    let label_style = look.typography_scale(ShadcnTextSize::Sm);
    div()
        .w(px(580.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(div().typography_style(label_style).text_color(label_color).child(title))
        .child(
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
    look: &ShadcnLook,
    label_color: gpui::Hsla,
    title_color: gpui::Hsla,
    rows: [(&str, &str); 5],
) -> impl IntoElement {
    let title_style = look.typography_scale(ShadcnTextSize::Sm);
    let row_style = look.typography_scale(ShadcnTextSize::Xs);
    let mut telemetry = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().typography_style(title_style).text_color(title_color).child("Telemetry"));

    for (label, sizes) in rows {
        telemetry = telemetry.child(telemetry_row(row_style, label_color, label, sizes));
    }

    telemetry
}

fn telemetry_row(style: LumaTextStyle, color: gpui::Hsla, label: &str, sizes: &str) -> impl IntoElement {
    div().typography_style(style).text_color(color).child(format!("{label}: {sizes}"))
}

fn format_sizes_px(sizes_px: &[f32]) -> String {
    sizes_px.iter().map(|size| format!("{size:.0}px")).collect::<Vec<_>>().join(" / ")
}

fn percent_of_content(size_px: f32) -> f32 {
    size_px / DEMO_WIDTH.max(1.0) * 100.0
}

fn label_foreground(theme: &ShadcnLook) -> gpui::Hsla {
    theme.token_color("foreground").unwrap_or_else(|_| theme.chrome().body_text)
}

fn demo_label(text: &str, theme: &ShadcnLook) -> AnyElement {
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
