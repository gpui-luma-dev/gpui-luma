use gpui::{
    ClickEvent, Context, DragMoveEvent, EventEmitter, IntoElement, MouseDownEvent, MouseUpEvent, Pixels, Render,
    SharedString, Window, div, prelude::*, px,
};

use super::{
    model::{PaneRender, SplitViewModel, clamp_sidebar_width, effective_sidebar_width},
    template::SplitViewTemplateHandlers,
    SplitViewBuilder, SplitViewRenderModel,
};

#[derive(Clone, Debug)]
pub enum SplitViewEvent {
    ResizeStart,
    SidebarWidthChanged { width: Pixels },
    ResizeEnd { width: Pixels },
    CollapsedChanged { collapsed: bool },
}

#[derive(Clone, Debug)]
pub struct SplitViewSeparatorDrag {
    pub(crate) id: SharedString,
}

impl SplitViewSeparatorDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for SplitViewSeparatorDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub struct SplitView {
    model: SplitViewModel,
    separator_hovered: bool,
    dragging_separator: bool,
    drag_moved: bool,
    suppress_next_separator_click: bool,
    drag_start_axis_px: f32,
    drag_start_width: Pixels,
}

impl EventEmitter<SplitViewEvent> for SplitView {}

impl SplitView {
    const DRAG_CLICK_SUPPRESS_THRESHOLD_PX: f32 = 2.0;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SplitViewBuilder {
        SplitViewBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SplitViewBuilder, _cx: &mut Context<Self>) -> Self {
        Self {
            model: builder.model,
            separator_hovered: false,
            dragging_separator: false,
            drag_moved: false,
            suppress_next_separator_click: false,
            drag_start_axis_px: 0.0,
            drag_start_width: px(0.0),
        }
    }

    pub fn sidebar_width(&self) -> Pixels {
        self.model.sidebar_width
    }

    pub fn effective_sidebar_width(&self) -> Pixels {
        effective_sidebar_width(self.model.collapsed, self.model.sidebar_width, self.model.sidebar_collapsed_width)
    }

    pub fn collapsed(&self) -> bool {
        self.model.collapsed
    }

    pub fn separator_visibility(&self) -> super::SplitViewSeparatorVisibility {
        self.model.separator_visibility
    }

    pub fn set_separator_visibility(
        &mut self,
        visibility: super::SplitViewSeparatorVisibility,
        cx: &mut Context<Self>,
    ) {
        if self.model.separator_visibility == visibility {
            return;
        }

        self.model.separator_visibility = visibility;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::theme::SplitViewTheme>, cx: &mut Context<Self>) {
        if std::sync::Arc::ptr_eq(&self.model.theme, &theme) {
            cx.notify();
            return;
        }

        self.model.theme = theme;
        cx.notify();
    }

    pub fn set_panes(&mut self, sidebar: PaneRender, content: PaneRender, cx: &mut Context<Self>) {
        self.model.sidebar = sidebar;
        self.model.content = content;
        cx.notify();
    }

    pub fn set_sidebar_width(&mut self, width: Pixels, cx: &mut Context<Self>) {
        let width = clamp_sidebar_width(width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        if (self.model.sidebar_width.as_f32() - width.as_f32()).abs() <= f32::EPSILON {
            return;
        }

        self.model.sidebar_width = width;
        cx.emit(SplitViewEvent::SidebarWidthChanged { width });
        cx.notify();
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.model.collapsed == collapsed {
            return;
        }

        self.model.collapsed = collapsed;
        self.separator_hovered = false;
        self.dragging_separator = false;
        self.drag_moved = false;
        self.suppress_next_separator_click = false;
        cx.emit(SplitViewEvent::CollapsedChanged { collapsed });
        cx.notify();
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.set_collapsed(!self.model.collapsed, cx);
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        if !enabled {
            self.dragging_separator = false;
            self.drag_moved = false;
            self.suppress_next_separator_click = false;
        }
        cx.notify();
    }

    fn render_model(&self) -> SplitViewRenderModel<'_> {
        SplitViewRenderModel {
            id: &self.model.id,
            sidebar_width: self.model.sidebar_width,
            sidebar_collapsed_width: self.model.sidebar_collapsed_width,
            effective_sidebar_width: self.effective_sidebar_width(),
            collapsed: self.model.collapsed,
            resizable: self.model.resizable,
            enabled: self.model.enabled,
            separator_hovered: self.separator_hovered,
            separator_visibility: self.model.separator_visibility,
            theme: &self.model.theme,
            separator_color_override: self.model.separator_color_override,
            separator_hover_color_override: self.model.separator_hover_color_override,
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> SplitViewTemplateHandlers {
        SplitViewTemplateHandlers {
            separator_hover: Box::new(cx.listener(Self::handle_separator_hover)),
            separator_mouse_down: Box::new(cx.listener(Self::handle_separator_mouse_down)),
            separator_click: Box::new(cx.listener(Self::handle_separator_click)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up_out)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
        }
    }

    fn handle_separator_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.separator_hovered == *hovered {
            return;
        }

        self.separator_hovered = *hovered;
        cx.notify();
    }

    fn handle_separator_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.resizable || self.model.collapsed {
            return;
        }

        self.dragging_separator = true;
        self.drag_moved = false;
        self.suppress_next_separator_click = false;
        self.drag_start_axis_px = event.position.x.as_f32();
        self.drag_start_width = self.model.sidebar_width;
        cx.notify();
    }

    fn handle_separator_click(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if self.suppress_next_separator_click {
            self.suppress_next_separator_click = false;
            self.drag_moved = false;
            return;
        }

        self.toggle_collapsed(cx);
    }

    fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<SplitViewSeparatorDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.model.id || !self.dragging_separator {
            return;
        }

        let delta = event.event.position.x.as_f32() - self.drag_start_axis_px;
        if !self.drag_moved && delta.abs() < Self::DRAG_CLICK_SUPPRESS_THRESHOLD_PX {
            return;
        }

        if !self.drag_moved {
            self.drag_moved = true;
            cx.emit(SplitViewEvent::ResizeStart);
        }

        self.set_sidebar_width(self.drag_start_width + px(delta), cx);
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.dragging_separator {
            return;
        }

        self.dragging_separator = false;
        if self.drag_moved {
            self.suppress_next_separator_click = true;
            cx.emit(SplitViewEvent::ResizeEnd { width: self.model.sidebar_width });
            cx.notify();
        }
    }

    fn handle_mouse_up_out(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.dragging_separator {
            return;
        }

        self.dragging_separator = false;
        self.suppress_next_separator_click = false;
        if self.drag_moved {
            self.drag_moved = false;
            cx.emit(SplitViewEvent::ResizeEnd { width: self.model.sidebar_width });
            cx.notify();
        }
    }
}

impl Render for SplitView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = (self.model.sidebar)();
        let content = (self.model.content)();
        let template = self.model.template.clone();
        let model = self.render_model();
        let handlers = self.template_handlers(cx);

        div()
            .size_full()
            .child(template.render(&model, sidebar, content, handlers, window, cx))
            .into_any_element()
    }
}
