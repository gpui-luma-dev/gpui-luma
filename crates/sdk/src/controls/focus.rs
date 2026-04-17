use gpui::{Context, KeyDownEvent, Window};

pub(crate) fn blur_on_escape<T: 'static>(
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<T>,
) -> bool {
    if event.keystroke.key != "escape" || event.keystroke.modifiers.modified() {
        return false;
    }

    window.blur();
    cx.stop_propagation();
    cx.notify();
    true
}
