use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Component, Context, Decorations, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Render, RenderOnce, StatefulInteractiveElement as _, StyleRefinement, Styled,
    TitlebarOptions, Window, WindowControlArea, div, hsla, point, prelude::*, px,
};
use lucide_icons::Icon as LucideIcon;

pub const TITLE_BAR_HEIGHT: Pixels = px(34.0);

#[cfg(target_os = "macos")]
const TITLE_BAR_LEFT_PADDING: Pixels = px(80.0);
#[cfg(not(target_os = "macos"))]
const TITLE_BAR_LEFT_PADDING: Pixels = px(12.0);

type CloseWindowHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// TitleBar used to customize the appearance of the title bar.
///
/// You can put arbitrary elements inside the title bar.
pub struct TitleBar {
    style: StyleRefinement,
    children: Vec<AnyElement>,
    on_close_window: Option<CloseWindowHandler>,
    background_color: Option<Hsla>,
    border_color: Option<Hsla>,
}

impl TitleBar {
    /// Create a new TitleBar.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            children: Vec::new(),
            on_close_window: None,
            background_color: None,
            border_color: None,
        }
    }

    /// Default title bar options compatible with this `TitleBar`.
    pub fn title_bar_options() -> TitlebarOptions {
        TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }
    }

    /// Set custom close-window behavior.
    ///
    /// Linux only; on other platforms this is ignored.
    pub fn on_close_window(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        if cfg!(target_os = "linux") {
            self.on_close_window = Some(Rc::new(f));
        }
        self
    }

    /// Set a custom title bar background color.
    pub fn background_color(mut self, color: Hsla) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set a custom title bar bottom border color.
    pub fn border_color(mut self, color: Hsla) -> Self {
        self.border_color = Some(color);
        self
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for TitleBar {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

#[derive(Clone)]
enum ControlIcon {
    Minimize,
    Restore,
    Maximize,
    Close { on_close_window: Option<CloseWindowHandler> },
}

impl ControlIcon {
    fn minimize() -> Self {
        Self::Minimize
    }

    fn restore() -> Self {
        Self::Restore
    }

    fn maximize() -> Self {
        Self::Maximize
    }

    fn close(on_close_window: Option<CloseWindowHandler>) -> Self {
        Self::Close { on_close_window }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Restore => "restore",
            Self::Maximize => "maximize",
            Self::Close { .. } => "close",
        }
    }

    fn icon(&self) -> LucideIcon {
        match self {
            Self::Minimize => LucideIcon::Minus,
            Self::Restore => LucideIcon::Copy,
            Self::Maximize => LucideIcon::Square,
            Self::Close { .. } => LucideIcon::X,
        }
    }

    fn window_control_area(&self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Restore | Self::Maximize => WindowControlArea::Max,
            Self::Close { .. } => WindowControlArea::Close,
        }
    }

    fn is_close(&self) -> bool {
        matches!(self, Self::Close { .. })
    }

    #[inline]
    fn hover_fg(&self) -> Hsla {
        if self.is_close() {
            hsla(0.0, 0.0, 1.0, 1.0)
        } else {
            hsla(0.0, 0.0, 1.0, 0.95)
        }
    }

    #[inline]
    fn hover_bg(&self) -> Hsla {
        if self.is_close() {
            hsla(0.0, 0.78, 0.56, 1.0)
        } else {
            hsla(0.0, 0.0, 1.0, 0.12)
        }
    }

    #[inline]
    fn active_bg(&self) -> Hsla {
        if self.is_close() {
            hsla(0.0, 0.78, 0.50, 1.0)
        } else {
            hsla(0.0, 0.0, 1.0, 0.2)
        }
    }
}

fn render_control_icon(icon: ControlIcon) -> AnyElement {
    let is_linux = cfg!(target_os = "linux");
    let is_windows = cfg!(target_os = "windows");

    let hover_fg = icon.hover_fg();
    let hover_bg = icon.hover_bg();
    let active_bg = icon.active_bg();
    let icon_for_click = icon.clone();
    let on_close_window = match &icon {
        ControlIcon::Close { on_close_window } => on_close_window.clone(),
        _ => None,
    };

    div()
        .id(icon.id())
        .flex()
        .w(TITLE_BAR_HEIGHT)
        .h_full()
        .flex_shrink_0()
        .justify_center()
        .items_center()
        .text_color(hsla(0.0, 0.0, 1.0, 0.85))
        .hover(|style| style.bg(hover_bg).text_color(hover_fg))
        .active(|style| style.bg(active_bg).text_color(hover_fg))
        .when(is_windows, |this| this.window_control_area(icon.window_control_area()))
        .when(is_linux, |this| {
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                match &icon_for_click {
                    ControlIcon::Minimize => window.minimize_window(),
                    ControlIcon::Restore | ControlIcon::Maximize => window.zoom_window(),
                    ControlIcon::Close { .. } => {
                        if let Some(f) = on_close_window.as_ref() {
                            f(&ClickEvent::default(), window, cx);
                        } else {
                            window.remove_window();
                        }
                    }
                }
            })
        })
        .child(render_lucide_icon(icon.icon(), 16.0))
        .into_any_element()
}

fn render_window_controls(window: &mut Window, on_close_window: Option<CloseWindowHandler>) -> AnyElement {
    if cfg!(target_os = "macos") {
        return div().id("window-controls").into_any_element();
    }

    div()
        .id("window-controls")
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .h_full()
        .child(render_control_icon(ControlIcon::minimize()))
        .child(render_control_icon(if window.is_maximized() {
            ControlIcon::restore()
        } else {
            ControlIcon::maximize()
        }))
        .child(render_control_icon(ControlIcon::close(on_close_window)))
        .into_any_element()
}

impl Styled for TitleBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

struct TitleBarState {
    should_move: bool,
}

// Keep this lightweight state render to satisfy `use_state` requirements.
impl Render for TitleBarState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_client_decorated = matches!(window.window_decorations(), Decorations::Client { .. });
        let is_linux = cfg!(target_os = "linux");
        let border_color = self.border_color.unwrap_or(hsla(0.0, 0.0, 1.0, 0.1));
        let background_color = self.background_color.unwrap_or(hsla(0.0, 0.0, 0.11, 1.0));

        let state = window.use_state(cx, |_, _| TitleBarState { should_move: false });

        div().flex_shrink_0().child(
            div()
                .id("title-bar")
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(TITLE_BAR_HEIGHT)
                .pl(TITLE_BAR_LEFT_PADDING)
                .border_b_1()
                .border_color(border_color)
                .bg(background_color)
                .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
                    state.should_move = false;
                }))
                .on_mouse_down(
                    MouseButton::Left,
                    window.listener_for(&state, |state, _, _, _| {
                        state.should_move = true;
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    window.listener_for(&state, |state, _, _, _| {
                        state.should_move = false;
                    }),
                )
                .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
                    if state.should_move {
                        state.should_move = false;
                        window.start_window_move();
                    }
                }))
                .on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                    if ev.click_count >= 2 {
                        cx.stop_propagation();
                        if !window.is_fullscreen() {
                            window.zoom_window();
                        }
                    }
                })
                .child(
                    div()
                        .id("bar")
                        .window_control_area(WindowControlArea::Drag)
                        .when(window.is_fullscreen(), |this| this.pl_3())
                        .h_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .flex_shrink_0()
                        .flex_1()
                        .when(is_linux && is_client_decorated, |this| {
                            this.child(
                                div()
                                    .top_0()
                                    .left_0()
                                    .absolute()
                                    .size_full()
                                    .h_full()
                                    .on_mouse_down(MouseButton::Right, move |ev, window, _| {
                                        window.show_window_menu(ev.position)
                                    }),
                            )
                        })
                        .children(self.children),
                )
                .child(render_window_controls(window, self.on_close_window)),
        )
    }
}

fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .text_size(px(size))
        .line_height(px(size))
        .child(char::from(icon).to_string())
        .into_any_element()
}
