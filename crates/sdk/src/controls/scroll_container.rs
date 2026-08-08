use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, EntityId, Pixels, ScrollHandle, ScrollWheelEvent,
    SharedString, Stateful, Task, Window, div, point, prelude::*, px,
};

use crate::controls::scrollbar::{Scrollbar, ScrollbarTemplate};

type ScrollWheelHandler = Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>;

/// Whether the scrollbar reserves layout space or floats over content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarPlacement {
    /// Reserve a gutter beside the viewport (default).
    #[default]
    Inset,
    /// Float the scrollbar over the viewport without reserving layout space.
    Overlay,
}

/// When the scrollbar chrome is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Always show the scrollbar while content is scrollable (default).
    #[default]
    AlwaysVisible,
    /// Show according to [`ScrollbarAutoHideActivate`]; hide when idle / not hovered.
    AutoHide,
    /// Never show the scrollbar chrome.
    Hidden,
}

/// What reveals an auto-hiding scrollbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarAutoHideActivate {
    /// Reveal while the pointer hovers the container.
    Hover,
    /// Reveal while scrolling (mouse wheel / trackpad); hide after motion stops.
    Move,
    /// Reveal on hover or scroll motion (default).
    #[default]
    HoverOrMove,
}

impl ScrollbarAutoHideActivate {
    fn listens_to_hover(self) -> bool {
        matches!(self, Self::Hover | Self::HoverOrMove)
    }

    fn listens_to_move(self) -> bool {
        matches!(self, Self::Move | Self::HoverOrMove)
    }
}

const AUTO_HIDE_TIMEOUT: Duration = Duration::from_millis(1250);

#[derive(Clone)]
pub struct ScrollContainer {
    id: SharedString,
    scroll_handle: ScrollHandle,
    scrollbar: Entity<Scrollbar>,
    scrollbar_width: Pixels,
    placement: ScrollbarPlacement,
    visibility: ScrollbarVisibility,
    auto_hide_activate: ScrollbarAutoHideActivate,
    last_max_scroll: Rc<Cell<f32>>,
    host_view: Rc<Cell<Option<EntityId>>>,
    hovered: Rc<Cell<bool>>,
    active_until: Rc<Cell<Option<Instant>>>,
    hide_task: Rc<RefCell<Option<Task<()>>>>,
}

impl ScrollContainer {
    pub fn new(
        id: impl Into<SharedString>,
        scrollbar_template: Arc<dyn ScrollbarTemplate>,
        cx: &mut impl AppContext,
    ) -> Self {
        let id = id.into();
        let scrollbar = Scrollbar::new(format!("{id}-scrollbar")).vertical().template(scrollbar_template).spawn(cx);

        Self {
            id,
            scroll_handle: ScrollHandle::new(),
            scrollbar,
            scrollbar_width: px(12.0),
            placement: ScrollbarPlacement::Inset,
            visibility: ScrollbarVisibility::AlwaysVisible,
            auto_hide_activate: ScrollbarAutoHideActivate::HoverOrMove,
            last_max_scroll: Rc::new(Cell::new(0.0)),
            host_view: Rc::new(Cell::new(None)),
            hovered: Rc::new(Cell::new(false)),
            active_until: Rc::new(Cell::new(None)),
            hide_task: Rc::new(RefCell::new(None)),
        }
    }

    pub fn placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.placement = if overlay {
            ScrollbarPlacement::Overlay
        } else {
            ScrollbarPlacement::Inset
        };
        self
    }

    pub fn visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.visibility = if auto_hide {
            ScrollbarVisibility::AutoHide
        } else {
            ScrollbarVisibility::AlwaysVisible
        };
        self
    }

    pub fn auto_hide_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.auto_hide_activate = activate;
        self
    }

    pub fn scrollbar_placement(&self) -> ScrollbarPlacement {
        self.placement
    }

    pub fn scrollbar_visibility(&self) -> ScrollbarVisibility {
        self.visibility
    }

    pub fn scrollbar_auto_hide_activate(&self) -> ScrollbarAutoHideActivate {
        self.auto_hide_activate
    }

    pub fn scrollbar(&self) -> Entity<Scrollbar> {
        self.scrollbar.clone()
    }

    pub fn set_vertical_offset<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    pub fn vertical_offset(&self) -> Pixels {
        px((-self.scroll_handle.offset().y.as_f32()).max(0.0))
    }

    pub fn max_vertical_offset(&self) -> Pixels {
        px(self.scroll_handle.max_offset().y.as_f32().max(0.0))
    }

    pub fn scroll_vertical_by<T: 'static>(&self, delta: Pixels, cx: &mut Context<T>) -> bool {
        let current = self.vertical_offset().as_f32();
        let target = (current + delta.as_f32()).clamp(0.0, self.max_vertical_offset().as_f32());
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn ensure_vertical_bounds_visible<T: 'static>(&self, top: Pixels, bottom: Pixels, cx: &mut Context<T>) -> bool {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let current = self.vertical_offset().as_f32();
        let mut target = current;

        let visible_top = current;
        let visible_bottom = current + viewport_height;

        let top = top.as_f32();
        let bottom = bottom.as_f32();

        if top < visible_top {
            target -= visible_top - top;
        } else if bottom > visible_bottom {
            target += bottom - visible_bottom;
        }

        target = target.clamp(0.0, max_scroll);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn sync_scrollbar<T: 'static>(&self, cx: &mut Context<T>) {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let value = (-self.scroll_handle.offset().y.as_f32()).clamp(0.0, max_scroll);
        let content_height = viewport_height + max_scroll;

        self.scrollbar.update(cx, |scrollbar, cx| {
            scrollbar.set_length(viewport_height.max(1.0), cx);
            scrollbar.set_step(24.0, cx);
            scrollbar.set_page_step((viewport_height * 0.85).max(1.0), cx);
            scrollbar.set_viewport(0.0, content_height.max(1.0), value, value + viewport_height, cx);
            scrollbar.set_enabled(max_scroll > 0.5, cx);
        });
    }

    pub fn render(&self, content: AnyElement) -> Stateful<gpui::Div> {
        self.render_internal(content, None)
    }

    pub fn render_with_scroll_wheel(
        &self,
        content: AnyElement,
        on_scroll_wheel: impl Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<gpui::Div> {
        self.render_internal(content, Some(Box::new(on_scroll_wheel)))
    }

    fn render_internal(&self, content: AnyElement, on_scroll_wheel: Option<ScrollWheelHandler>) -> Stateful<gpui::Div> {
        let scrollable = self.scroll_handle.max_offset().y.as_f32() > 0.5;
        let viewport_right = viewport_right_inset(scrollable, self.placement, self.visibility, self.scrollbar_width);
        let scroll_handle = self.scroll_handle.clone();
        let last_max_scroll = self.last_max_scroll.clone();
        let host_view = self.host_view.clone();
        let auto_hide = self.visibility == ScrollbarVisibility::AutoHide;
        let activate = self.auto_hide_activate;
        let track_hover = auto_hide && activate.listens_to_hover();
        let track_move = auto_hide && activate.listens_to_move();
        let show_scrollbar = scrollable
            && self.visibility != ScrollbarVisibility::Hidden
            && scrollbar_chrome_visible(
                self.visibility,
                activate,
                self.hovered.get(),
                self.active_until.get(),
                Instant::now(),
            );

        let mut viewport = div()
            .on_children_prepainted({
                let host_view = host_view.clone();
                move |_: Vec<Bounds<Pixels>>, window: &mut Window, cx: &mut App| {
                    // `current_view` is only valid during layout/prepaint/paint.
                    host_view.set(Some(window.current_view()));
                    let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                    if (last_max_scroll.get() - max_scroll).abs() > 0.5 {
                        last_max_scroll.set(max_scroll);
                        notify_host(&host_view, cx);
                    }
                }
            })
            .id(format!("{}-viewport", self.id))
            .absolute()
            .top(px(0.0))
            .bottom(px(0.0))
            .left(px(0.0))
            .right(viewport_right)
            .overflow_y_scroll()
            .scrollbar_width(px(0.0))
            .track_scroll(&self.scroll_handle)
            .child(content);

        if track_move || on_scroll_wheel.is_some() {
            let active_until = self.active_until.clone();
            let hide_task = self.hide_task.clone();
            let host_view = host_view.clone();
            viewport = viewport.on_scroll_wheel(move |event, window, cx| {
                if track_move {
                    wake_on_move(&active_until, &hide_task, &host_view, cx);
                }
                if let Some(handler) = &on_scroll_wheel {
                    handler(event, window, cx);
                }
            });
        }

        let mut root = div().id(self.id.clone()).relative().size_full();

        if track_hover {
            let hovered = self.hovered.clone();
            let active_until = self.active_until.clone();
            let hide_task = self.hide_task.clone();
            let host_view = host_view.clone();
            root = root.on_hover(move |is_hovered, _window, cx| {
                let is_hovered = *is_hovered;
                if hovered.get() == is_hovered {
                    return;
                }
                hovered.set(is_hovered);
                if is_hovered {
                    // Hover keeps chrome visible; cancel a pending move-idle hide.
                    *hide_task.borrow_mut() = None;
                } else if activate == ScrollbarAutoHideActivate::Hover {
                    // Hover-only: hide as soon as the pointer leaves.
                    active_until.set(None);
                    *hide_task.borrow_mut() = None;
                }
                notify_host(&host_view, cx);
            });
        }

        root.child(viewport).when(show_scrollbar, |root| {
            root.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .right(px(0.0))
                    .bottom(px(0.0))
                    .w(self.scrollbar_width)
                    .flex()
                    .justify_center()
                    .child(self.scrollbar.clone()),
            )
        })
    }
}

fn viewport_right_inset(
    scrollable: bool,
    placement: ScrollbarPlacement,
    visibility: ScrollbarVisibility,
    scrollbar_width: Pixels,
) -> Pixels {
    if scrollable && placement == ScrollbarPlacement::Inset && visibility != ScrollbarVisibility::Hidden {
        scrollbar_width
    } else {
        px(0.0)
    }
}

fn scrollbar_chrome_visible(
    visibility: ScrollbarVisibility,
    activate: ScrollbarAutoHideActivate,
    hovered: bool,
    active_until: Option<Instant>,
    now: Instant,
) -> bool {
    match visibility {
        ScrollbarVisibility::AlwaysVisible => true,
        ScrollbarVisibility::Hidden => false,
        ScrollbarVisibility::AutoHide => {
            let hover_active = activate.listens_to_hover() && hovered;
            let move_active = activate.listens_to_move() && active_until.is_some_and(|until| now < until);
            hover_active || move_active
        }
    }
}

fn notify_host(host_view: &Rc<Cell<Option<EntityId>>>, cx: &mut App) {
    if let Some(view) = host_view.get() {
        cx.notify(view);
    }
}

fn wake_on_move(
    active_until: &Rc<Cell<Option<Instant>>>,
    hide_task: &Rc<RefCell<Option<Task<()>>>>,
    host_view: &Rc<Cell<Option<EntityId>>>,
    cx: &mut App,
) {
    schedule_auto_hide(active_until, hide_task, host_view, cx);
    notify_host(host_view, cx);
}

fn schedule_auto_hide(
    active_until: &Rc<Cell<Option<Instant>>>,
    hide_task: &Rc<RefCell<Option<Task<()>>>>,
    host_view: &Rc<Cell<Option<EntityId>>>,
    cx: &mut App,
) {
    let until = Instant::now() + AUTO_HIDE_TIMEOUT;
    active_until.set(Some(until));
    let active_until = active_until.clone();
    let host_view = host_view.clone();
    *hide_task.borrow_mut() = Some(cx.spawn(async move |cx| {
        cx.background_executor().timer(AUTO_HIDE_TIMEOUT).await;
        cx.update(|cx| {
            if active_until.get().is_some_and(|deadline| Instant::now() >= deadline) {
                active_until.set(None);
                notify_host(&host_view, cx);
            }
        });
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_inset_always_visible_and_hover_or_move() {
        assert_eq!(ScrollbarPlacement::default(), ScrollbarPlacement::Inset);
        assert_eq!(ScrollbarVisibility::default(), ScrollbarVisibility::AlwaysVisible);
        assert_eq!(ScrollbarAutoHideActivate::default(), ScrollbarAutoHideActivate::HoverOrMove);
    }

    #[test]
    fn overlay_placement_never_reserves_gutter() {
        assert_eq!(
            viewport_right_inset(true, ScrollbarPlacement::Overlay, ScrollbarVisibility::AlwaysVisible, px(12.0)),
            px(0.0)
        );
        assert_eq!(
            viewport_right_inset(true, ScrollbarPlacement::Overlay, ScrollbarVisibility::AutoHide, px(12.0)),
            px(0.0)
        );
    }

    #[test]
    fn inset_placement_reserves_gutter_when_scrollable() {
        assert_eq!(
            viewport_right_inset(true, ScrollbarPlacement::Inset, ScrollbarVisibility::AlwaysVisible, px(12.0)),
            px(12.0)
        );
        assert_eq!(
            viewport_right_inset(true, ScrollbarPlacement::Inset, ScrollbarVisibility::AutoHide, px(12.0)),
            px(12.0)
        );
        assert_eq!(
            viewport_right_inset(false, ScrollbarPlacement::Inset, ScrollbarVisibility::AlwaysVisible, px(12.0)),
            px(0.0)
        );
    }

    #[test]
    fn hidden_visibility_never_reserves_gutter() {
        assert_eq!(
            viewport_right_inset(true, ScrollbarPlacement::Inset, ScrollbarVisibility::Hidden, px(12.0)),
            px(0.0)
        );
    }

    #[test]
    fn hover_activate_ignores_move_deadline() {
        let now = Instant::now();
        assert!(scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::Hover,
            true,
            None,
            now
        ));
        assert!(!scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::Hover,
            false,
            Some(now + Duration::from_millis(500)),
            now
        ));
    }

    #[test]
    fn move_activate_ignores_hover() {
        let now = Instant::now();
        assert!(!scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::Move,
            true,
            None,
            now
        ));
        assert!(scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::Move,
            false,
            Some(now + Duration::from_millis(500)),
            now
        ));
        assert!(!scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::Move,
            true,
            Some(now - Duration::from_millis(1)),
            now
        ));
    }

    #[test]
    fn hover_or_move_accepts_either_signal() {
        let now = Instant::now();
        assert!(scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::HoverOrMove,
            true,
            None,
            now
        ));
        assert!(scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::HoverOrMove,
            false,
            Some(now + Duration::from_millis(500)),
            now
        ));
        assert!(!scrollbar_chrome_visible(
            ScrollbarVisibility::AutoHide,
            ScrollbarAutoHideActivate::HoverOrMove,
            false,
            None,
            now
        ));
    }

    #[test]
    fn always_visible_and_hidden_chrome_are_constant() {
        let now = Instant::now();
        assert!(scrollbar_chrome_visible(
            ScrollbarVisibility::AlwaysVisible,
            ScrollbarAutoHideActivate::Move,
            false,
            None,
            now
        ));
        assert!(!scrollbar_chrome_visible(
            ScrollbarVisibility::Hidden,
            ScrollbarAutoHideActivate::HoverOrMove,
            true,
            Some(now + Duration::from_secs(1)),
            now
        ));
    }
}
