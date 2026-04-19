use gpui::{App, FocusHandle, InteractiveElement, KeyBinding, Window, actions};

actions!(luma_focus, [NextFocus, PreviousFocus, EscapeFocus]);

const FOCUS_CONTEXT: &str = "LumaFocus";

pub fn bind_default_focus_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", NextFocus, Some(FOCUS_CONTEXT)),
        KeyBinding::new("shift-tab", PreviousFocus, Some(FOCUS_CONTEXT)),
        KeyBinding::new("escape", EscapeFocus, Some(FOCUS_CONTEXT)),
    ]);
}

pub trait LumaFocusScopeExt: InteractiveElement + Sized {
    fn luma_focus_scope(self, scope: &FocusHandle) -> Self;
}

impl<E> LumaFocusScopeExt for E
where
    E: InteractiveElement + Sized,
{
    fn luma_focus_scope(self, scope: &FocusHandle) -> Self {
        let scope = scope.clone();

        self.track_focus(&scope)
            .key_context(FOCUS_CONTEXT)
            .on_action(|_: &NextFocus, window: &mut Window, cx: &mut App| {
                window.focus_next(cx);
            })
            .on_action(|_: &PreviousFocus, window: &mut Window, cx: &mut App| {
                window.focus_prev(cx);
            })
            .on_action(move |_: &EscapeFocus, window: &mut Window, cx: &mut App| {
                window.focus(&scope, cx);
            })
    }
}
