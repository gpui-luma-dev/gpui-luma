use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, Hsla, IntoElement, Pixels, SharedString, div, px, prelude::*};
use gpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterEvent, DockSplitterTheme, SplitterOrientation, ThemedDockSplitterTemplate,
};
use gpui_luma::dock_panel;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma_look_shadcn::ShadcnLook;

type SidebarWidthChanged = Rc<dyn Fn(Pixels)>;

#[derive(Clone)]
pub struct WorkbenchSidebar {
    render: Rc<dyn Fn() -> AnyElement>,
    width_px: f32,
    min_px: f32,
    max_px: f32,
    on_width_changed: Option<SidebarWidthChanged>,
}

impl WorkbenchSidebar {
    pub fn new(render: impl Fn() -> AnyElement + 'static) -> Self {
        Self { render: Rc::new(render), width_px: 360.0, min_px: 360.0, max_px: 460.0, on_width_changed: None }
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width_px = width.as_f32();
        self
    }

    pub fn min(mut self, min: Pixels) -> Self {
        self.min_px = min.as_f32();
        self
    }

    pub fn max(mut self, max: Pixels) -> Self {
        self.max_px = max.as_f32();
        self
    }

    pub fn on_width_changed(mut self, callback: impl Fn(Pixels) + 'static) -> Self {
        self.on_width_changed = Some(Rc::new(callback));
        self
    }
}

pub struct WorkbenchLayout {
    content: Rc<dyn Fn() -> AnyElement>,
    left_sidebar: WorkbenchSidebar,
    right_sidebar: WorkbenchSidebar,
    left_width_px: Rc<Cell<f32>>,
    right_width_px: Rc<Cell<f32>>,
    left_splitter: gpui::Entity<DockSplitter>,
    right_splitter: gpui::Entity<DockSplitter>,
}

impl WorkbenchLayout {
    pub fn new<T: 'static>(
        id: impl Into<SharedString>,
        look: Arc<ShadcnLook>,
        left_sidebar: WorkbenchSidebar,
        content: impl Fn() -> AnyElement + 'static,
        right_sidebar: WorkbenchSidebar,
        cx: &mut Context<T>,
    ) -> Self {
        let id = id.into();
        let content: Rc<dyn Fn() -> AnyElement> = Rc::new(content);
        let splitter_theme = look.dock_splitter_theme();
        let splitter_template = Arc::new(ThemedDockSplitterTemplate::new(true));

        let left_splitter = DockSplitter::new(format!("{id}-left-splitter"), SplitterOrientation::Vertical)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let right_splitter = DockSplitter::new(format!("{id}-right-splitter"), SplitterOrientation::Vertical)
            .template(splitter_template)
            .theme(splitter_theme)
            .spawn(cx);

        let left_width_px = Rc::new(Cell::new(left_sidebar.width_px));
        let left_drag_start_px = Rc::new(Cell::new(left_sidebar.width_px));
        let right_width_px = Rc::new(Cell::new(right_sidebar.width_px));
        let right_drag_start_px = Rc::new(Cell::new(right_sidebar.width_px));

        let left_min_px = left_sidebar.min_px;
        let left_max_px = left_sidebar.max_px;
        let left_width_changed = left_sidebar.on_width_changed.clone();
        let left_width_px_for_events = left_width_px.clone();
        let left_drag_start_px_for_events = left_drag_start_px.clone();
        cx.subscribe(&left_splitter, move |_, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => {
                left_drag_start_px_for_events.set(left_width_px_for_events.get());
            }
            DockSplitterEvent::Resize { total_delta } => {
                let next = (left_drag_start_px_for_events.get() + *total_delta).clamp(left_min_px, left_max_px);
                if (left_width_px_for_events.get() - next).abs() <= f32::EPSILON {
                    return;
                }
                left_width_px_for_events.set(next);
                if let Some(callback) = &left_width_changed {
                    callback(px(next));
                }
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();

        let right_min_px = right_sidebar.min_px;
        let right_max_px = right_sidebar.max_px;
        let right_width_changed = right_sidebar.on_width_changed.clone();
        let right_width_px_for_events = right_width_px.clone();
        let right_drag_start_px_for_events = right_drag_start_px.clone();
        cx.subscribe(&right_splitter, move |_, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => {
                right_drag_start_px_for_events.set(right_width_px_for_events.get());
            }
            DockSplitterEvent::Resize { total_delta } => {
                let next = (right_drag_start_px_for_events.get() - *total_delta).clamp(right_min_px, right_max_px);
                if (right_width_px_for_events.get() - next).abs() <= f32::EPSILON {
                    return;
                }
                right_width_px_for_events.set(next);
                if let Some(callback) = &right_width_changed {
                    callback(px(next));
                }
                cx.notify();
            }
            DockSplitterEvent::ResizeEnd => {}
        })
        .detach();

        Self { content, left_sidebar, right_sidebar, left_width_px, right_width_px, left_splitter, right_splitter }
    }

    pub fn sync_theme<T>(&self, theme: &Arc<dyn DockSplitterTheme>, cx: &mut Context<T>) {
        self.left_splitter.update(cx, |splitter, cx| splitter.set_theme(theme.clone(), cx));
        self.right_splitter.update(cx, |splitter, cx| splitter.set_theme(theme.clone(), cx));
    }

    pub fn render_body(&self, left_collapsed: bool, right_collapsed: bool) -> AnyElement {
        let body = render_panel_slot(self.content.clone());

        match (left_collapsed, right_collapsed) {
            (false, false) => dock_panel! {
                left: render_sidebar(self.left_sidebar.render.clone(), self.left_width_px.get()),
                left: self.left_splitter.clone(),
                right: render_sidebar(self.right_sidebar.render.clone(), self.right_width_px.get()),
                right: self.right_splitter.clone(),
                fill: body
            }
            .into_any_element(),
            (false, true) => dock_panel! {
                left: render_sidebar(self.left_sidebar.render.clone(), self.left_width_px.get()),
                left: self.left_splitter.clone(),
                fill: body
            }
            .into_any_element(),
            (true, false) => dock_panel! {
                right: render_sidebar(self.right_sidebar.render.clone(), self.right_width_px.get()),
                right: self.right_splitter.clone(),
                fill: body
            }
            .into_any_element(),
            (true, true) => body,
        }
    }

    pub fn render_shell(
        focus_scope: &FocusHandle,
        font_family: SharedString,
        app_background: Hsla,
        title_bar: impl IntoElement,
        body: impl IntoElement,
        bottom_bar: impl IntoElement,
    ) -> AnyElement {
        div()
            .luma_focus_scope(focus_scope)
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .font_family(font_family)
            .bg(app_background)
            .child(title_bar)
            .child(div().id("workbench-layout-body").flex_1().min_h_0().w_full().overflow_hidden().child(body))
            .child(bottom_bar)
            .into_any_element()
    }
}

fn render_panel_slot(render: Rc<dyn Fn() -> AnyElement>) -> AnyElement {
    div().size_full().min_h_0().flex().flex_col().overflow_hidden().child(render()).into_any_element()
}

fn render_sidebar(render: Rc<dyn Fn() -> AnyElement>, width_px: f32) -> AnyElement {
    div()
        .w(px(width_px))
        .h_full()
        .min_h_0()
        .flex_shrink_0()
        .overflow_hidden()
        .child(render())
        .into_any_element()
}
