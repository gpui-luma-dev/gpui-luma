use super::*;

fn extended() -> ListBoxState<u32, u32> {
    ListBoxState::from_snapshot(
        ListBoxSnapshot::try_with_enabled(1..=6, |n| *n, |n| *n != 3).unwrap(),
        SelectionMode::Extended,
    )
}

fn selected(list: &ListBoxState<u32, u32>) -> Vec<u32> {
    list.selected_keys().copied().collect()
}

fn gesture(key: u32, toggle: bool, extend: bool) -> ListBoxInput<u32> {
    ListBoxInput::SelectWithModifiers { key, modifiers: ListBoxSelectionModifiers { toggle, extend } }
}

#[test]
fn ranges_extend_contract_reverse_and_skip_disabled_items() {
    let mut list = extended();
    list.apply(ListBoxInput::Select(2));
    let update = list.apply(gesture(6, false, true));
    assert_eq!(selected(&list), vec![2, 4, 5, 6]);
    assert_eq!(list.anchor_key(), Some(&2));
    assert_eq!(
        update.events,
        vec![
            ListBoxEvent::SelectionChanged { selected: vec![2, 4, 5, 6], active: Some(6) },
            ListBoxEvent::ActiveItemChanged { active: Some(6) },
        ]
    );
    list.apply(gesture(4, false, true));
    assert_eq!(selected(&list), vec![2, 4]);
    list.apply(gesture(1, false, true));
    assert_eq!(selected(&list), vec![1, 2]);
    assert_eq!(list.anchor_key(), Some(&2));
    assert!(!list.apply(gesture(1, false, true)).changed);
    for invalid in [3, 99] {
        assert!(!list.apply(gesture(invalid, true, true)).changed);
        assert_eq!(list.anchor_key(), Some(&2));
        assert_eq!(list.active_key(), Some(&1));
    }
}

#[test]
fn toggle_sets_anchor_and_additive_range_keeps_other_selections() {
    let mut list = extended();
    list.apply(ListBoxInput::Select(6));
    list.apply(gesture(2, true, false));
    list.apply(gesture(4, true, true));
    assert_eq!(selected(&list), vec![2, 4, 6]);
    assert_eq!(list.anchor_key(), Some(&2));
    list.apply(gesture(2, true, false));
    assert_eq!(selected(&list), vec![4, 6]);
    assert_eq!(list.anchor_key(), Some(&2));
    list.apply(ListBoxInput::Select(5));
    assert_eq!(selected(&list), vec![5]);
}

#[test]
fn extended_plain_toggle_off_is_opt_in_and_preserves_ranges_and_confirmation() {
    let mut list = extended();
    list.apply(ListBoxInput::Select(2));
    assert!(!list.apply(ListBoxInput::Select(2)).changed); // SDK default remains ordinary Extended.
    list.set_selection_policy(SelectionPolicy { toggle_off: true, ..list.selection_policy() });
    list.apply(gesture(5, true, false));
    let update = list.apply(ListBoxInput::Select(5));
    assert_eq!(selected(&list), vec![2]); // Only the clicked item is unchecked.
    assert_eq!(list.active_key(), Some(&5));
    assert_eq!(list.anchor_key(), Some(&5));
    assert_eq!(update.events, vec![ListBoxEvent::SelectionChanged { selected: vec![2], active: Some(5) }]);
    list.apply(gesture(2, false, true)); // Shift on a checked item still extends a range.
    assert_eq!(selected(&list), vec![2, 4, 5]);
    list.apply(ListBoxInput::ConfirmActive); // Enter never toggles off a checked item.
    assert_eq!(selected(&list), vec![2, 4, 5]);
    list.apply(ListBoxInput::SelectActive); // Space follows plain toggle-off policy.
    assert_eq!(selected(&list), vec![4, 5]);
    list.apply(ListBoxInput::Select(1)); // A new plain target still replaces the selection.
    assert_eq!(selected(&list), vec![1]);
    list.apply(ListBoxInput::Select(1));
    assert!(selected(&list).is_empty());
    list.set_selection_policy(SelectionPolicy { toggle_off: false, ..list.selection_policy() });
    list.apply(ListBoxInput::Select(1));
    assert!(!list.apply(ListBoxInput::Select(1)).changed);
}

#[test]
fn keyboard_ranges_reveal_and_use_pre_gesture_active_after_anchor_reset() {
    let mut list = extended();
    list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
    let update = list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Next, additive: false });
    assert_eq!(selected(&list), vec![2, 4]);
    assert_eq!(update.reveal, Some(4));
    list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Previous, additive: false });
    assert_eq!(selected(&list), vec![2]);
    list.apply(ListBoxInput::ClearSelection);
    assert_eq!(list.anchor_key(), None);
    list.apply(gesture(5, false, true));
    assert_eq!(selected(&list), vec![2, 4, 5]);
    list.apply(ListBoxInput::SelectAll);
    assert_eq!(selected(&list), vec![1, 2, 4, 5, 6]);
    assert_eq!(list.anchor_key(), None);
    list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Last, additive: false });
    assert_eq!(selected(&list), vec![5, 6]);
    assert!(
        !list
            .apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Next, additive: false })
            .changed
    );
}

#[test]
fn anchor_survives_reorder_but_is_cleared_on_disable_remove_or_explicit_selection() {
    let mut list = extended();
    list.apply(ListBoxInput::Select(2));
    list.replace_snapshot(ListBoxSnapshot::try_new([6, 5, 4, 2, 1], |n| *n).unwrap());
    list.apply(gesture(5, false, true));
    assert_eq!(selected(&list), vec![5, 4, 2]);
    assert_eq!(list.anchor_key(), Some(&2));
    assert_eq!(list.set_selected(Some(99)).unwrap_err(), ListBoxError::UnknownKey);
    assert_eq!(list.anchor_key(), Some(&2));
    list.set_selected(Some(6)).unwrap();
    assert_eq!(list.anchor_key(), None);
    for disable in [false, true] {
        list.apply(ListBoxInput::Select(2));
        let items = if disable { vec![6, 5, 4, 2, 1] } else { vec![6, 5, 4, 1] };
        list.replace_snapshot(ListBoxSnapshot::try_with_enabled(items, |n| *n, |n| *n != 2).unwrap());
        assert_eq!(list.anchor_key(), None);
        list.replace_snapshot(ListBoxSnapshot::try_new([6, 5, 4, 2, 1], |n| *n).unwrap());
    }
}

#[test]
fn no_selection_keeps_navigation_and_activation_and_rejects_programmatic_selection() {
    let mut list = extended();
    list.apply(ListBoxInput::Select(2));
    list.set_selection_mode(SelectionMode::None);
    list.apply(gesture(5, true, true));
    assert_eq!(list.active_key(), Some(&5));
    assert!(selected(&list).is_empty());
    assert_eq!(list.set_selected(Some(2)).unwrap_err(), ListBoxError::SelectionDisabled);
    let snapshot = ListBoxSnapshot::try_new([7, 8], |n| *n).unwrap();
    assert_eq!(
        list.replace_snapshot_with_selection(snapshot, [7], Some(7)).unwrap_err(),
        ListBoxError::SelectionDisabled
    );
    assert_eq!(list.snapshot().items(), &[1, 2, 3, 4, 5, 6]);
    assert!(!list.apply(ListBoxInput::SelectAll).changed);
    assert!(!list.apply(ListBoxInput::SelectActive).changed);
    assert_eq!(list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next)).reveal, Some(6));
    assert_eq!(list.apply(ListBoxInput::ActivateActive).events, vec![ListBoxEvent::ItemActivated { key: 6 }]);
}

#[test]
fn runtime_policy_changes_reconcile_once_preserve_focus_and_do_not_recreate_state() {
    let mut list = extended();
    list.apply(ListBoxInput::Focus(true));
    list.apply(ListBoxInput::Select(2));
    list.apply(gesture(5, false, true));
    let policy = SelectionPolicy::new(SelectionMode::SingleAllowNone);
    assert_eq!(
        list.set_selection_policy(policy).events,
        vec![
            ListBoxEvent::SelectionPolicyChanged { policy },
            ListBoxEvent::SelectionChanged { selected: vec![5], active: Some(5) },
        ]
    );
    assert_eq!(list.anchor_key(), None);
    assert!(list.is_focused());
    assert!(!list.set_selection_policy(policy).changed);
    list.set_selection_mode(SelectionMode::Multiple);
    list.set_selected_keys([1, 2]).unwrap();
    list.set_selection_mode(SelectionMode::SingleRequired);
    assert_eq!(selected(&list), vec![1]); // Active is not among selected keys.
    list.set_selection_mode(SelectionMode::None);
    list.set_selection_mode(SelectionMode::SingleRequired);
    assert_eq!(selected(&list), vec![5]); // Required fills from active.
    list.replace_snapshot(ListBoxSnapshot::try_new([], |n: &u32| *n).unwrap());
    list.set_selection_mode(SelectionMode::None);
    list.set_selection_mode(SelectionMode::SingleRequired);
    assert!(selected(&list).is_empty());
}

#[test]
fn single_toggle_off_and_follow_navigation_obey_required_selection() {
    for mode in [SelectionMode::SingleAllowNone, SelectionMode::SingleRequired] {
        let mut list = extended();
        list.set_selection_policy(SelectionPolicy { mode, toggle_off: true, selection_follows_active: true });
        list.apply(ListBoxInput::Select(2));
        list.apply(ListBoxInput::Select(2));
        assert_eq!(
            selected(&list),
            if mode == SelectionMode::SingleRequired {
                vec![2]
            } else {
                vec![]
            }
        );
        let update = list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
        assert_eq!(update.reveal, Some(4));
        assert_eq!(
            update.events,
            vec![
                ListBoxEvent::SelectionChanged { selected: vec![4], active: Some(4) },
                ListBoxEvent::ActiveItemChanged { active: Some(4) },
            ]
        );
        let mut policy = list.selection_policy();
        policy.selection_follows_active = false;
        list.set_selection_policy(policy);
        list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
        assert_eq!(selected(&list), vec![4]);
    }
    let mut list = extended();
    list.set_selection_policy(SelectionPolicy {
        mode: SelectionMode::Multiple,
        toggle_off: true,
        selection_follows_active: true,
    });
    list.apply(ListBoxInput::Select(2));
    list.apply(ListBoxInput::Navigate(ListBoxNavigation::Next));
    assert_eq!(selected(&list), vec![2]);
}

#[test]
fn drag_capture_preserves_source_order_and_never_mutates_selection() {
    let mut list = extended();
    list.set_selected_keys([6, 2, 4]).unwrap();
    let captured = list.drag_keys(&4).unwrap();
    assert_eq!(captured, vec![2, 4, 6]);
    assert_eq!(list.drag_keys(&5), Some(vec![5]));
    assert_eq!(list.drag_keys(&3), None);
    assert_eq!(list.drag_keys(&99), None);
    assert_eq!(selected(&list), vec![2, 4, 6]);
    list.apply(ListBoxInput::ClearSelection);
    assert_eq!(captured, vec![2, 4, 6]);
    list.set_selection_mode(SelectionMode::None);
    assert_eq!(list.drag_keys(&4), Some(vec![4]));
}
