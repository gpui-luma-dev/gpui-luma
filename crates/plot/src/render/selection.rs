use gpui::{App, Window};
use crate::views::selection::SelectionEvents;
use crate::SelectionEvent;
struct Hover<T> {
    target: Option<T>,
}
pub(super) fn report<T: Clone + PartialEq + 'static>(
    target: Option<T>,
    events: &SelectionEvents<T>,
    window: &mut Window,
    cx: &mut App,
) {
    let state = window.use_state(cx, |_, _| Hover::<T> { target: None });
    let changed = state.update(cx, |state, _| {
        let changed = state.target != target;
        state.target = target.clone();
        changed
    });
    if changed {
        let events = events.clone();
        cx.defer(move |cx| (events.0)(SelectionEvent::HoverChanged { target }, cx));
    }
}
