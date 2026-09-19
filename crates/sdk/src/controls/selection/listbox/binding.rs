use gpui::{
    App, ClickEvent, Context, Div, FocusHandle, KeyDownEvent, MouseButton, Role, Stateful, Subscription, Window,
    prelude::*,
};

use super::{ListBoxInput, ListBoxItemState, ListBoxNavigation, SelectionMode};

/// Physical key mapping belongs to the binding, not the collection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListBoxAxis {
    Vertical,
    Horizontal,
}

type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);

/// Focus and input wiring for a host-owned list surface. Keep one binding per
/// list for its lifetime. Nested interactive children must stop consumed pointer
/// events; keyboard events are handled only while the list itself holds focus.
pub struct ListBoxBinding {
    focus: FocusHandle,
    mode: SelectionMode,
    subscriptions: Vec<Subscription>,
}

impl ListBoxBinding {
    /// Use the same selection mode as the bound collection state.
    pub fn new(mode: SelectionMode, cx: &mut App) -> Self {
        Self { focus: cx.focus_handle().tab_stop(true), mode, subscriptions: Vec::new() }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// Attach to a stable, host-owned viewport. Register focus subscriptions on
    /// first render, then reuse them for the lifetime of the owning view.
    pub fn bind_root<M: 'static, K: 'static>(
        &mut self,
        root: Stateful<Div>,
        axis: ListBoxAxis,
        window: &mut Window,
        cx: &mut Context<M>,
        on_input: InputHandler<M, K>,
    ) -> Stateful<Div> {
        if self.subscriptions.is_empty() {
            self.subscriptions.push(cx.on_focus_in(&self.focus, window, move |owner, window, cx| {
                on_input(owner, ListBoxInput::Focus(true), window, cx);
            }));
            self.subscriptions.push(cx.on_focus_out(&self.focus, window, move |owner, _, window, cx| {
                on_input(owner, ListBoxInput::Focus(false), window, cx);
            }));
        }
        let focus = self.focus.clone();
        let mode = self.mode;
        root.track_focus(&self.focus)
            .role(Role::ListBox)
            .aria_orientation(match axis {
                ListBoxAxis::Vertical => gpui::Orientation::Vertical,
                ListBoxAxis::Horizontal => gpui::Orientation::Horizontal,
            })
            .on_key_down(cx.listener(move |owner, event: &KeyDownEvent, window, cx| {
                if !focus.is_focused(window) {
                    return;
                }
                let Some(input) = key_input(mode, axis, &event.keystroke.key, event.keystroke.modifiers) else {
                    return;
                };
                on_input(owner, input, window, cx);
                cx.stop_propagation();
            }))
    }

    /// Attach to a look-styled row with a stable element ID derived from its
    /// list identity and key. Content and dimensions remain host choices.
    pub fn bind_row<M: 'static, K: Clone + 'static>(
        &self,
        row: Stateful<Div>,
        key: K,
        state: ListBoxItemState,
        cx: &mut Context<M>,
        on_input: InputHandler<M, K>,
    ) -> Stateful<Div> {
        let row = row
            .role(Role::ListBoxOption)
            .aria_selected(state.selected)
            .when(state.active, |row| row.aria_active_descendant());
        if !state.enabled {
            return row;
        }
        let focus = self.focus.clone();
        row.on_mouse_down(MouseButton::Left, move |_, window, cx| {
            focus.focus(window, cx);
            cx.stop_propagation();
        })
        .on_click(cx.listener(move |owner, event: &ClickEvent, window, cx| {
            if event.is_keyboard() {
                return;
            }
            let input = if event.click_count() == 2 {
                ListBoxInput::Activate(key.clone())
            } else {
                ListBoxInput::Select(key.clone())
            };
            on_input(owner, input, window, cx);
            cx.stop_propagation();
        }))
    }
}

/// Translate only list-owned keys; leave other shortcuts available to ancestors.
fn key_input<K>(
    mode: SelectionMode,
    axis: ListBoxAxis,
    key: &str,
    modifiers: gpui::Modifiers,
) -> Option<ListBoxInput<K>> {
    if mode == SelectionMode::Multiple
        && key.eq_ignore_ascii_case("a")
        && (modifiers.platform || modifiers.control)
        && !modifiers.alt
        && !modifiers.function
    {
        return Some(if modifiers.shift {
            ListBoxInput::ClearSelection
        } else {
            ListBoxInput::SelectAll
        });
    }
    if modifiers.modified() {
        return None;
    }
    Some(match key {
        "up" if axis == ListBoxAxis::Vertical => ListBoxInput::Navigate(ListBoxNavigation::Previous),
        "down" if axis == ListBoxAxis::Vertical => ListBoxInput::Navigate(ListBoxNavigation::Next),
        "left" if axis == ListBoxAxis::Horizontal => ListBoxInput::Navigate(ListBoxNavigation::Previous),
        "right" if axis == ListBoxAxis::Horizontal => ListBoxInput::Navigate(ListBoxNavigation::Next),
        "home" => ListBoxInput::Navigate(ListBoxNavigation::First),
        "end" => ListBoxInput::Navigate(ListBoxNavigation::Last),
        "space" => ListBoxInput::SelectActive,
        "enter" => ListBoxInput::ActivateActive,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Modifiers;

    #[test]
    fn selection_shortcuts_are_scoped_to_multiple_mode() {
        for modifiers in [
            Modifiers { platform: true, ..Default::default() },
            Modifiers { control: true, ..Default::default() },
        ] {
            assert!(matches!(
                key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "a", modifiers),
                Some(ListBoxInput::SelectAll)
            ));
            assert!(key_input::<u32>(SelectionMode::SingleAllowNone, ListBoxAxis::Vertical, "a", modifiers).is_none());
            let clear_modifiers = Modifiers { shift: true, ..modifiers };
            assert!(matches!(
                key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "a", clear_modifiers),
                Some(ListBoxInput::ClearSelection)
            ));
            assert!(
                key_input::<u32>(SelectionMode::SingleAllowNone, ListBoxAxis::Vertical, "a", clear_modifiers).is_none()
            );
        }
        // Escape belongs to the enclosing focus scope, even in multiple mode.
        assert!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "escape", Modifiers::default())
                .is_none()
        );
        assert!(
            key_input::<u32>(SelectionMode::SingleRequired, ListBoxAxis::Vertical, "escape", Modifiers::default())
                .is_none()
        );
        assert!(
            key_input::<u32>(
                SelectionMode::Multiple,
                ListBoxAxis::Vertical,
                "a",
                Modifiers { control: true, alt: true, ..Default::default() }
            )
            .is_none()
        );
    }

    #[test]
    fn clear_shortcut_removes_selection_without_losing_focus_or_active_item() {
        use super::super::{ListBoxEvent, ListBoxState};

        for axis in [ListBoxAxis::Vertical, ListBoxAxis::Horizontal] {
            let mut list = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
            list.apply(ListBoxInput::Focus(true));
            list.apply(ListBoxInput::Select(2));
            list.apply(ListBoxInput::SelectAll);
            let input = key_input(
                SelectionMode::Multiple,
                axis,
                "A",
                Modifiers { platform: true, shift: true, ..Default::default() },
            )
            .unwrap();
            let update = list.apply(input);
            assert_eq!(update.events, vec![ListBoxEvent::SelectionChanged { selected: vec![], active: Some(2) }]);
            assert_eq!(list.selected_keys().count(), 0);
            assert!(list.is_focused());
            assert_eq!(list.active_key(), Some(&2));
            assert!(!list.apply(ListBoxInput::ClearSelection).changed);
        }
    }

    #[test]
    fn navigation_uses_each_lists_axis_without_consuming_unrelated_keys() {
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "right", Modifiers::default()),
            Some(ListBoxInput::Navigate(ListBoxNavigation::Next))
        ));
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "down", Modifiers::default()),
            Some(ListBoxInput::Navigate(ListBoxNavigation::Next))
        ));
        assert!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "down", Modifiers::default()).is_none()
        );
        assert!(
            key_input::<u32>(
                SelectionMode::Multiple,
                ListBoxAxis::Vertical,
                "down",
                Modifiers { shift: true, ..Default::default() }
            )
            .is_none()
        );
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "space", Modifiers::default()),
            Some(ListBoxInput::SelectActive)
        ));
    }
}
