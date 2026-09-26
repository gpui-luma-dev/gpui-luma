use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{App, EntityId, Pixels, Task, px};

use super::model::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

pub(crate) const AUTO_HIDE_TIMEOUT: Duration = Duration::from_millis(1250);

pub(super) fn viewport_right_inset(
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

pub(super) fn scrollbar_chrome_visible(
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

pub(super) fn notify_host(host_view: &Rc<Cell<Option<EntityId>>>, cx: &mut App) {
    if let Some(view) = host_view.get() {
        cx.notify(view);
    }
}

pub(super) fn wake_on_move(
    active_until: &Rc<Cell<Option<Instant>>>,
    hide_task: &Rc<RefCell<Option<Task<()>>>>,
    host_view: &Rc<Cell<Option<EntityId>>>,
    cx: &mut App,
) {
    schedule_auto_hide(active_until, hide_task, host_view, cx);
    notify_host(host_view, cx);
}

pub(super) fn schedule_auto_hide(
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
    use super::super::model::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

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
