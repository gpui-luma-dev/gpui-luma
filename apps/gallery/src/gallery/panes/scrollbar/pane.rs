use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Context, DragMoveEvent, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    ScrollWheelEvent, SharedString, Subscription, Window, div, prelude::*, px, rgb,
};
use gpui_luma::controls::scrollbar::{
    Scrollbar, ScrollbarBoundsHandler, ScrollbarDrag, ScrollbarDragMoveHandler, ScrollbarEvent, ScrollbarHoverHandler,
    ScrollbarMouseDownHandler, ScrollbarMouseUpHandler, ScrollbarOrientation, ScrollbarRenderModel,
    ScrollbarScrollWheelHandler, ScrollbarTemplate, ScrollbarTemplateHandlers,
};
use gpui_luma::controls::value::ControlRange;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_scrollbar_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ScrollbarPane {
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    state_preview: Entity<ScrollbarStatePreview>,
    inspector: Entity<ColorInspectorShell>,
    horizontal_value: f32,
    vertical_value: f32,
}

#[derive(Clone, Copy)]
enum ScrollbarPresentation {
    Horizontal,
    Vertical,
}

impl ScrollbarPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let scrollbar_template = look.scrollbar_template();
        let tree =
            spawn_color_inspector_tree("scrollbar-inspector-tree", look.clone(), build_scrollbar_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "scrollbar-inspector",
                "scrollbar-inspector-split",
                "scrollbar-inspector-detail",
                build_scrollbar_inspect_tree,
                cx,
            )
        });
        Self {
            horizontal_scrollbar: look
                .scrollbar("scrollbar-horizontal-example")
                .horizontal()
                .range(0..220)
                .step(20)
                .page_step(80)
                .value(40)
                .thumb_fraction(0.54)
                .spawn(cx),
            vertical_scrollbar: look
                .scrollbar("scrollbar-vertical-example")
                .vertical()
                .range(0..240)
                .step(20)
                .page_step(80)
                .value(80)
                .thumb_fraction(0.45)
                .spawn(cx),
            state_preview: cx.new(|_| ScrollbarStatePreview::new(look.clone(), scrollbar_template)),
            inspector,
            horizontal_value: 40.0,
            vertical_value: 80.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.horizontal_scrollbar, |app, _, event: &ScrollbarEvent, cx| {
            app.panes.scrollbar.handle_event(ScrollbarPresentation::Horizontal, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_scrollbar, |app, _, event: &ScrollbarEvent, cx| {
            app.panes.scrollbar.handle_event(ScrollbarPresentation::Vertical, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_inspector(
            "Scrollbar",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .child(self.scrollbar_example(look))
                .child(self.state_preview.clone())
                .into_any_element(),
            self.inspector.clone(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.horizontal_scrollbar, cx);
        notify_entity(&self.vertical_scrollbar, cx);
        notify_entity(&self.state_preview, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_event(
        &mut self,
        presentation: ScrollbarPresentation,
        event: &ScrollbarEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            ScrollbarEvent::Change { value } => {
                match presentation {
                    ScrollbarPresentation::Horizontal => {
                        self.horizontal_value = *value;
                    }
                    ScrollbarPresentation::Vertical => {
                        self.vertical_value = *value;
                    }
                }
                cx.notify();
            }
        }
    }

    fn scrollbar_example(&self, look: &ShadcnLook) -> AnyElement {
        scrollbar_pair(
            self.horizontal_value,
            self.vertical_value,
            self.horizontal_scrollbar.clone(),
            self.vertical_scrollbar.clone(),
            look,
        )
    }
}

#[derive(Clone)]
struct ScrollbarStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ScrollbarTemplate>,
}

struct ScrollbarStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ScrollbarStatePreview {
    fn new(look: Arc<ShadcnLook>, template: Arc<dyn ScrollbarTemplate>) -> Self {
        Self { look, template }
    }
}

impl Render for ScrollbarStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ScrollbarStateSample { id: "default", label: "Standard", state: InteractionState::default() },
            ScrollbarStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ScrollbarStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ScrollbarStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ScrollbarStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
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
                "Horizontal",
                ScrollbarOrientation::Horizontal,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
            .child(render_state_row(
                &self.template,
                "Vertical",
                ScrollbarOrientation::Vertical,
                &samples,
                chrome.muted_text,
                window,
                cx,
            ))
    }
}

fn render_state_row(
    template: &Arc<dyn ScrollbarTemplate>,
    row_label: &'static str,
    orientation: ScrollbarOrientation,
    samples: &[ScrollbarStateSample],
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
                    .map(|sample| render_state_sample(template, orientation, sample, label_color, window, cx)),
            ),
        )
        .into_any_element()
}

fn render_state_sample(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: &ScrollbarStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let orientation_id = match orientation {
        ScrollbarOrientation::Horizontal => "horizontal",
        ScrollbarOrientation::Vertical => "vertical",
    };
    let id = SharedString::from(format!("scrollbar-preview-{}-{}", orientation_id, sample.id));
    let range = ControlRange::from(0..220);
    let value = 40.0;
    let model = ScrollbarRenderModel {
        id: &id,
        orientation,
        range,
        step: 20.0,
        page_step: 80.0,
        value,
        percentage: range.percentage(value),
        thumb_fraction: 0.54,
        length: None,
        enabled: !sample.state.disabled,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, scrollbar_preview_handlers(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn scrollbar_preview_handlers() -> ScrollbarTemplateHandlers {
    ScrollbarTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as ScrollbarBoundsHandler,
        thumb_bounds: Box::new(noop_bounds) as ScrollbarBoundsHandler,
        hover: Box::new(noop_hover) as ScrollbarHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as ScrollbarMouseDownHandler,
        mouse_up: Box::new(noop_mouse_up) as ScrollbarMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as ScrollbarMouseUpHandler,
        drag_move: Box::new(noop_drag_move) as ScrollbarDragMoveHandler,
        scroll_wheel: Box::new(noop_scroll_wheel) as ScrollbarScrollWheelHandler,
    }
}

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_drag_move(_: &DragMoveEvent<ScrollbarDrag>, _: &mut Window, _: &mut App) {}

fn noop_scroll_wheel(_: &ScrollWheelEvent, _: &mut Window, _: &mut App) {}

fn scrollbar_pair(
    horizontal_value: f32,
    vertical_value: f32,
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();
    let mut demo_content = div()
        .absolute()
        .left(px(-horizontal_value))
        .top(px(-vertical_value))
        .flex()
        .flex_col()
        .gap_2()
        .p_3();

    for row in 0..12 {
        demo_content = demo_content.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().w(px(92.0)).text_color(chrome.title_text).child(format!("Row {:02}", row + 1)))
                .child(div().w(px(110.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbae6fd)))
                .child(div().w(px(150.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbbf7d0)))
                .child(div().w(px(96.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xfed7aa))),
        );
    }

    div()
        .flex()
        .items_start()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .relative()
                        .w(px(260.0))
                        .h(px(180.0))
                        .overflow_hidden()
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(chrome.border)
                        .bg(chrome.panel_background)
                        .child(demo_content),
                )
                .child(horizontal_scrollbar),
        )
        .child(vertical_scrollbar)
        .into_any_element()
}
