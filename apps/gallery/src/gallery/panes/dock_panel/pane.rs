use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Focusable, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::dock_splitter::{DockSplitter, DockSplitterEvent, SplitterOrientation, ThemedDockSplitterTemplate};
use gpui_luma::dock_panel;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::super::shared::notify_entity;

#[derive(Clone)]
pub(in crate::gallery) struct DockPanelPane {
    state: Entity<DockPanelPaneState>,
}

impl DockPanelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let state = cx.new(|cx| DockPanelPaneState::new(look, cx));
        Self { state }
    }

    pub(in crate::gallery) fn render(&self, _look: &ShadcnLook) -> AnyElement {
        self.state.clone().into_any_element()
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn notify_controls(&self, window: &mut Window, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(window, cx));
        notify_entity(&self.state, cx);
    }
}

struct DockPanelPaneState {
    look: Arc<ShadcnLook>,
    left_width: f32,
    right_width: f32,
    top_height: f32,
    bottom_height: f32,
    drag_start_left_width: f32,
    drag_start_right_width: f32,
    drag_start_top_height: f32,
    drag_start_bottom_height: f32,
    left_splitter: Entity<DockSplitter>,
    top_splitter: Entity<DockSplitter>,
    right_splitter: Entity<DockSplitter>,
    bottom_splitter: Entity<DockSplitter>,
}

impl DockPanelPaneState {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let splitter_theme = look.dock_splitter_theme();
        let splitter_template = Arc::new(ThemedDockSplitterTemplate::new(true));
        let left_splitter = DockSplitter::new("dock-panel-left-splitter", SplitterOrientation::Vertical)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let top_splitter = DockSplitter::new("dock-panel-top-splitter", SplitterOrientation::Horizontal)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let right_splitter = DockSplitter::new("dock-panel-right-splitter", SplitterOrientation::Vertical)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let bottom_splitter = DockSplitter::new("dock-panel-bottom-splitter", SplitterOrientation::Horizontal)
            .template(splitter_template)
            .theme(splitter_theme)
            .spawn(cx);

        let this = Self {
            look,
            left_width: 80.0,
            right_width: 80.0,
            top_height: 50.0,
            bottom_height: 50.0,
            drag_start_left_width: 80.0,
            drag_start_right_width: 80.0,
            drag_start_top_height: 50.0,
            drag_start_bottom_height: 50.0,
            left_splitter,
            top_splitter,
            right_splitter,
            bottom_splitter,
        };

        cx.subscribe(&this.left_splitter, |this, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start_left_width = this.left_width,
            DockSplitterEvent::Resize { total_delta } => {
                this.left_width = (this.drag_start_left_width + *total_delta).clamp(40.0, 300.0);
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();
        cx.subscribe(&this.top_splitter, |this, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start_top_height = this.top_height,
            DockSplitterEvent::Resize { total_delta } => {
                this.top_height = (this.drag_start_top_height + *total_delta).clamp(30.0, 180.0);
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();
        cx.subscribe(&this.right_splitter, |this, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start_right_width = this.right_width,
            DockSplitterEvent::Resize { total_delta } => {
                this.right_width = (this.drag_start_right_width - *total_delta).clamp(40.0, 300.0);
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();
        cx.subscribe(&this.bottom_splitter, |this, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start_bottom_height = this.bottom_height,
            DockSplitterEvent::Resize { total_delta } => {
                this.bottom_height = (this.drag_start_bottom_height - *total_delta).clamp(30.0, 180.0);
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();

        this
    }

    fn notify_controls(&self, window: &mut Window, cx: &mut Context<Self>) {
        let left_splitter_focus = self.left_splitter.read(cx).focus_handle(cx);
        left_splitter_focus.focus(window, cx);

        self.left_splitter.update(cx, |_, cx| cx.notify());
        self.top_splitter.update(cx, |_, cx| cx.notify());
        self.right_splitter.update(cx, |_, cx| cx.notify());
        self.bottom_splitter.update(cx, |_, cx| cx.notify());
    }
}

impl Render for DockPanelPaneState {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let foreground = self.look.token_color("foreground").unwrap_or(chrome.body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let band_background = self.look.token_color("muted").unwrap_or(chrome.content_background);
        let fill_background = self.look.token_color("accent").unwrap_or(chrome.panel_background);
        let fill_foreground = self.look.token_color("accent-foreground").unwrap_or(foreground);

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.content_background)
            .p(px(28.0))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(20.0))
                            .line_height(px(28.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(chrome.title_text)
                            .child("DockSplitter"),
                    )
                    .child(
                        div()
                            .max_w(px(760.0))
                            .text_size(px(13.0))
                            .line_height(px(18.0))
                            .text_color(chrome.muted_text)
                            .child("Debug view for ordered docking geometry with resizable docked boundaries."),
                    ),
            )
            .child(div().flex_1().min_w(px(0.0)).min_h(px(0.0)).p(px(100.0)).child(
                div().size_full().overflow_hidden().border_1().border_color(chrome.border).child(dock_panel! {
                    left: div()
                        .w(px(self.left_width))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(foreground)
                        .child("Left"),
                    left: self.left_splitter.clone(),
                    top: div()
                        .h(px(self.top_height))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(foreground)
                        .child("Top"),
                    top: self.top_splitter.clone(),
                    right: div()
                        .w(px(self.right_width))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(foreground)
                        .child("Right"),
                    right: self.right_splitter.clone(),
                    bottom: div()
                        .h(px(self.bottom_height))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(foreground)
                        .child("Bottom"),
                    bottom: self.bottom_splitter.clone(),
                    fill: render_nested_dock_panel_examples(
                        chrome,
                        foreground,
                        muted,
                        band_background,
                        fill_background,
                        fill_foreground,
                    )
                }),
            ))
    }
}

fn render_nested_dock_panel_examples(
    chrome: gpui_luma::theme::LumaChrome,
    foreground: gpui::Hsla,
    muted: gpui::Hsla,
    band_background: gpui::Hsla,
    fill_background: gpui::Hsla,
    fill_foreground: gpui::Hsla,
) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(16.0))
        .p(px(24.0))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(
                    div()
                        .text_color(foreground)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("Nested DockPanel tests"),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(muted)
                        .child("Inner examples verify child ordering and last-child-fill behavior."),
                ),
        )
        .child(
            div().flex().gap(px(16.0)).children([
                render_nested_example_card(
                    chrome,
                    foreground,
                    muted,
                    "Default last child fills",
                    dock_panel! {
                        top: demo_band("Top", foreground, band_background),
                        left: demo_rail("Left", foreground, band_background),
                        child: demo_fill("Fill", fill_foreground, fill_background)
                    }
                    .into_any_element(),
                ),
                render_nested_example_card(
                    chrome,
                    foreground,
                    muted,
                    "last_child_fill = false",
                    dock_panel! {
                        last_child_fill = false;
                        top: demo_band("Top", foreground, band_background),
                        child: demo_rail("Child → Left", foreground, band_background),
                        right: demo_rail("Right", foreground, band_background)
                    }
                    .into_any_element(),
                ),
            ]),
        )
}

fn render_nested_example_card(
    chrome: gpui_luma::theme::LumaChrome,
    foreground: gpui::Hsla,
    muted: gpui::Hsla,
    title: &'static str,
    content: AnyElement,
) -> gpui::Div {
    div()
        .w(px(220.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(foreground)
                .child(title),
        )
        .child(
            div()
                .h(px(170.0))
                .w_full()
                .overflow_hidden()
                .rounded(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .child(content),
        )
        .child(div().text_size(px(11.0)).line_height(px(14.0)).text_color(muted).child(match title {
            "Default last child fills" => "The final child becomes the fill region.",
            _ => "Without fill, undocked child content falls back to Left docking.",
        }))
}

fn demo_band(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.0))
        .text_color(foreground)
        .bg(background)
        .child(label)
}

fn demo_rail(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .w(px(68.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.0))
        .text_color(foreground)
        .bg(background)
        .child(label)
}

fn demo_fill(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(foreground)
        .bg(background)
        .child(label)
}
