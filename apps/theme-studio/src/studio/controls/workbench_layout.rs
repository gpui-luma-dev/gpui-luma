use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, Hsla, IntoElement, Pixels, SharedString, Size, div, px, prelude::*};
use gpui_luma::controls::resizable_panels::{ResizablePanels, ResizablePanelsTheme};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::resizable_panels;
use gpui_luma_look_shadcn::ShadcnLook;

#[derive(Clone)]
pub struct WorkbenchSidebar {
    render: Rc<dyn Fn() -> AnyElement>,
    width_px: f32,
    min_px: f32,
    max_px: f32,
}

impl WorkbenchSidebar {
    pub fn new(render: impl Fn() -> AnyElement + 'static) -> Self {
        Self { render: Rc::new(render), width_px: 360.0, min_px: 360.0, max_px: 460.0 }
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
}

pub struct WorkbenchLayout {
    content: Rc<dyn Fn() -> AnyElement>,
    main_split: gpui::Entity<ResizablePanels>,
    right_split: gpui::Entity<ResizablePanels>,
    full_split: gpui::Entity<ResizablePanels>,
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

        let left_render = left_sidebar.render.clone();
        let content_for_main = content.clone();
        let main_split = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: format!("{id}-main-split"),
            layout: Horizontal,
            show_handle: true,
            resize_handle: Sm,
            handle_grip: true,
            show_border: false,
            panels: [
                move || render_panel_slot(left_render.clone()) => px(left_sidebar.width_px), min: px(left_sidebar.min_px), max: px(left_sidebar.max_px);
                |
                move || render_panel_slot(content_for_main.clone()) => weight(1.0);
            ]
        };

        let content_for_right = content.clone();
        let right_render = right_sidebar.render.clone();
        let right_split = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: format!("{id}-right-split"),
            layout: Horizontal,
            show_handle: true,
            resize_handle: Sm,
            handle_grip: true,
            show_border: false,
            panels: [
                move || render_panel_slot(content_for_right.clone()) => weight(1.0);
                |
                move || render_panel_slot(right_render.clone()) => px(right_sidebar.width_px), min: px(right_sidebar.min_px), max: px(right_sidebar.max_px);
            ]
        };

        let left_render_for_full = left_sidebar.render.clone();
        let content_for_full = content.clone();
        let right_render_for_full = right_sidebar.render.clone();
        let full_split = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: format!("{id}-full-split"),
            layout: Horizontal,
            show_handle: true,
            resize_handle: Sm,
            handle_grip: true,
            show_border: false,
            panels: [
                move || render_panel_slot(left_render_for_full.clone()) => px(left_sidebar.width_px), min: px(left_sidebar.min_px), max: px(left_sidebar.max_px);
                |
                move || render_panel_slot(content_for_full.clone()) => weight(1.0);
                |
                move || render_panel_slot(right_render_for_full.clone()) => px(right_sidebar.width_px), min: px(right_sidebar.min_px), max: px(right_sidebar.max_px);
            ]
        };

        Self { content, main_split, right_split, full_split }
    }

    pub fn sync_theme<T>(&self, theme: Arc<dyn ResizablePanelsTheme>, cx: &mut Context<T>) {
        self.main_split.update(cx, |split, cx| split.set_theme(theme.clone(), cx));
        self.right_split.update(cx, |split, cx| split.set_theme(theme.clone(), cx));
        self.full_split.update(cx, |split, cx| split.set_theme(theme, cx));
    }

    pub fn sync_measured_size<T>(&self, size: Size<Pixels>, cx: &mut Context<T>) {
        self.main_split.update(cx, |split, cx| split.set_measured_size(size, cx));
        self.right_split.update(cx, |split, cx| split.set_measured_size(size, cx));
        self.full_split.update(cx, |split, cx| split.set_measured_size(size, cx));
    }

    pub fn render_body(&self, left_collapsed: bool, right_collapsed: bool) -> AnyElement {
        match (left_collapsed, right_collapsed) {
            (false, false) => self.full_split.clone().into_any_element(),
            (false, true) => self.main_split.clone().into_any_element(),
            (true, false) => self.right_split.clone().into_any_element(),
            (true, true) => render_panel_slot(self.content.clone()),
        }
    }

    pub fn render_shell(
        &self,
        focus_scope: &FocusHandle,
        font_family: SharedString,
        app_background: Hsla,
        title_bar: impl IntoElement,
        body: impl IntoElement,
        bottom_bar: impl IntoElement,
    ) -> AnyElement {
        let _ = self;

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
