use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, IntoElement, MouseButton, Render, Stateful, Subscription, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterAppearance, DockSplitterRenderModel, DockSplitterTemplate, DockSplitterTemplateHandlers,
    SplitterOrientation,
};
use gpui_luma::dock_panel;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::super::shared::notify_entity;

struct LeftSplitterTemplate {
    look: Arc<ShadcnLook>,
}

impl DockSplitterTemplate for LeftSplitterTemplate {
    fn render(
        &self,
        model: &DockSplitterRenderModel<'_>,
        appearance: &DockSplitterAppearance,
        handlers: DockSplitterTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let DockSplitterTemplateHandlers { hover, mouse_down, mouse_up, mouse_up_out } = handlers;
        let line_color = if model.dragging || model.hovered {
            appearance.hover_color
        } else {
            appearance.line_color
        };
        let accent = self.look.token_color("accent").unwrap_or(appearance.hover_color);
        let half_inset = ((appearance.hit_target_px - appearance.visible_line_px) * 0.5).max(0.0);

        let mut root = div().id(format!("{}-layout", model.id)).relative().flex_shrink_0();
        root = match model.orientation {
            SplitterOrientation::Vertical => root.w(px(appearance.visible_line_px)).h_full(),
            SplitterOrientation::Horizontal => root.h(px(appearance.visible_line_px)).w_full(),
        };

        let grip = div().absolute().inset_0().flex().justify_center().items_center().child(
            div()
                .rounded(px(8.0))
                .bg(if model.hovered || model.dragging {
                    accent
                } else {
                    accent.opacity(0.4)
                })
                .w(px(4.0))
                .h(px(36.0)),
        );

        let hit_target = match model.orientation {
            SplitterOrientation::Vertical => div()
                .id(model.id.clone())
                .absolute()
                .left(px(-half_inset))
                .top(px(0.0))
                .bottom(px(0.0))
                .w(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_col_resize())
                .child(
                    div()
                        .absolute()
                        .left(px(half_inset))
                        .top(px(0.0))
                        .bottom(px(0.0))
                        .w(px(appearance.visible_line_px))
                        .bg(line_color),
                )
                .child(grip),
            SplitterOrientation::Horizontal => div()
                .id(model.id.clone())
                .absolute()
                .top(px(-half_inset))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(appearance.hit_target_px))
                .on_hover(hover)
                .on_mouse_down(MouseButton::Left, mouse_down)
                .on_mouse_up(MouseButton::Left, mouse_up)
                .on_mouse_up_out(MouseButton::Left, mouse_up_out)
                .when(model.enabled, |this| this.cursor_row_resize())
                .child(
                    div()
                        .absolute()
                        .top(px(half_inset))
                        .left(px(0.0))
                        .right(px(0.0))
                        .h(px(appearance.visible_line_px))
                        .bg(line_color),
                ),
        };

        root.child(hit_target)
    }
}

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

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.state.update(cx, |state, cx| state.notify_controls(cx));
        notify_entity(&self.state, cx);
    }
}

struct DockPanelPaneState {
    look: Arc<ShadcnLook>,
    left_width: f32,
    right_width: f32,
    top_height: f32,
    bottom_height: f32,
    left_splitter: Entity<DockSplitter>,
    top_splitter: Entity<DockSplitter>,
    right_splitter: Entity<DockSplitter>,
    bottom_splitter: Entity<DockSplitter>,
}

impl DockPanelPaneState {
    fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let splitter_theme = look.dock_splitter_theme();
        let left_splitter = DockSplitter::new("dock-panel-left-splitter", SplitterOrientation::Vertical)
            .template(Arc::new(LeftSplitterTemplate { look: look.clone() }))
            .theme(splitter_theme.clone())
            .spawn(cx);
        let top_splitter = DockSplitter::new("dock-panel-top-splitter", SplitterOrientation::Horizontal)
            .theme(splitter_theme.clone())
            .spawn(cx);
        let right_splitter = DockSplitter::new("dock-panel-right-splitter", SplitterOrientation::Vertical)
            .theme(splitter_theme.clone())
            .spawn(cx);
        let bottom_splitter = DockSplitter::new("dock-panel-bottom-splitter", SplitterOrientation::Horizontal)
            .theme(splitter_theme)
            .spawn(cx);

        let this = Self {
            look,
            left_width: 80.0,
            right_width: 80.0,
            top_height: 50.0,
            bottom_height: 50.0,
            left_splitter,
            top_splitter,
            right_splitter,
            bottom_splitter,
        };

        cx.subscribe(
            &this.left_splitter,
            |this, _, event: &gpui_luma::controls::dock_splitter::DockSplitterEvent, cx| {
                if let gpui_luma::controls::dock_splitter::DockSplitterEvent::Resize { delta } = event {
                    this.left_width = (this.left_width + *delta).clamp(40.0, 300.0);
                    cx.notify();
                }
            },
        )
        .detach();
        cx.subscribe(
            &this.top_splitter,
            |this, _, event: &gpui_luma::controls::dock_splitter::DockSplitterEvent, cx| {
                if let gpui_luma::controls::dock_splitter::DockSplitterEvent::Resize { delta } = event {
                    this.top_height = (this.top_height + *delta).clamp(30.0, 180.0);
                    cx.notify();
                }
            },
        )
        .detach();
        cx.subscribe(
            &this.right_splitter,
            |this, _, event: &gpui_luma::controls::dock_splitter::DockSplitterEvent, cx| {
                if let gpui_luma::controls::dock_splitter::DockSplitterEvent::Resize { delta } = event {
                    this.right_width = (this.right_width - *delta).clamp(40.0, 300.0);
                    cx.notify();
                }
            },
        )
        .detach();
        cx.subscribe(
            &this.bottom_splitter,
            |this, _, event: &gpui_luma::controls::dock_splitter::DockSplitterEvent, cx| {
                if let gpui_luma::controls::dock_splitter::DockSplitterEvent::Resize { delta } = event {
                    this.bottom_height = (this.bottom_height - *delta).clamp(30.0, 180.0);
                    cx.notify();
                }
            },
        )
        .detach();

        this
    }

    fn notify_controls(&self, cx: &mut Context<Self>) {
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
                    fill: div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(foreground)
                        .child("Center/Fill")
                }),
            ))
    }
}
