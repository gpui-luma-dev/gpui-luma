use super::*;

fn list() -> ListBoxState<u32, u32> {
    ListBoxState::from_snapshot(
        ListBoxSnapshot::try_with_enabled(1..=6, |n| *n, |n| *n != 3).unwrap(),
        SelectionMode::Extended,
    )
}

fn visible(list: &ListBoxState<u32, u32>) -> Vec<u32> {
    list.visible_items().map(|item| item.key).collect()
}

fn selected(list: &ListBoxState<u32, u32>) -> Vec<u32> {
    list.selected_keys().copied().collect()
}

#[test]
fn projection_preserves_source_and_hidden_selection_and_reconciles_active_anchor() {
    let mut list = list();
    list.apply(ListBoxInput::Select(2));
    list.set_selected_keys([2, 4, 6]).unwrap();
    list.apply(ListBoxInput::Navigate(ListBoxNavigation::First));
    list.apply(ListBoxInput::Focus(true));
    let update = list.set_projection([6, 3, 4]).unwrap();
    assert_eq!(
        update.events,
        vec![
            ListBoxEvent::ProjectionChanged { visible_count: 3 },
            ListBoxEvent::ActiveItemChanged { active: Some(6) },
        ]
    );
    assert_eq!(update.reveal, None);
    assert_eq!(list.snapshot().items(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!(selected(&list), vec![2, 4, 6]);
    assert_eq!(list.anchor_key(), None);
    assert!(list.is_focused());
    assert_eq!(list.visible_index(&2), None);
    let indices: Vec<_> = list.visible_items().map(|item| (item.key, item.source_index, item.visible_index)).collect();
    assert_eq!(indices, vec![(6, 5, 0), (3, 2, 1), (4, 3, 2)]);
    assert!(!list.set_projection([6, 3, 4]).unwrap().changed);
    list.reset_projection();
    assert_eq!(selected(&list), vec![2, 4, 6]);
    assert_eq!(list.active_key(), Some(&6));
    assert!(!list.reset_projection().changed);
}

#[test]
fn projection_validation_and_combined_replacement_are_atomic() {
    let mut list = list();
    list.apply(ListBoxInput::Select(2));
    list.set_projection([2, 4]).unwrap();
    for (keys, error) in [(vec![2, 2], ListBoxError::DuplicateKey), (vec![4, 99], ListBoxError::UnknownKey)] {
        assert_eq!(list.set_projection(keys.clone()).unwrap_err(), error);
        assert_eq!(
            list.replace_snapshot_with_projection(ListBoxSnapshot::try_new([2, 4], |n| *n).unwrap(), Some(keys))
                .unwrap_err(),
            error
        );
        assert_eq!(visible(&list), vec![2, 4]);
        assert_eq!(list.snapshot().items(), &[1, 2, 3, 4, 5, 6]);
        assert_eq!(list.anchor_key(), Some(&2));
        assert_eq!(selected(&list), vec![2]);
    }
    assert_eq!(
        list.replace_snapshot_with_projection_and_selection(
            ListBoxSnapshot::try_new([7, 8], |n| *n).unwrap(),
            Some(vec![8]),
            [7],
            Some(7)
        )
        .unwrap_err(),
        ListBoxError::HiddenItem
    );
    assert_eq!(list.active_key(), Some(&2));
    let update = list
        .replace_snapshot_with_projection_and_selection(
            ListBoxSnapshot::try_new([7, 8, 9], |n| *n).unwrap(),
            Some(vec![9, 8]),
            [7, 8],
            None,
        )
        .unwrap();
    assert_eq!(
        update.events,
        vec![
            ListBoxEvent::ProjectionChanged { visible_count: 2 },
            ListBoxEvent::SelectionChanged { selected: vec![7, 8], active: Some(8) },
            ListBoxEvent::ActiveItemChanged { active: Some(8) },
        ]
    );
}

#[test]
fn projection_navigation_ranges_and_select_all_follow_visible_order() {
    let mut list = list();
    list.set_projection([6, 3, 4, 2]).unwrap();
    list.apply(ListBoxInput::Select(6));
    let update = list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Next, additive: false });
    assert_eq!(update.reveal, Some(4));
    assert_eq!(selected(&list), vec![4, 6]); // events retain source order
    assert_eq!(list.anchor_key(), Some(&6));
    list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Last, additive: false });
    assert_eq!(selected(&list), vec![2, 4, 6]);
    list.apply(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Previous, additive: false });
    assert_eq!(selected(&list), vec![4, 6]);
    list.set_selected_keys([1]).unwrap(); // programmatic selection can be hidden
    list.apply(ListBoxInput::SelectAll);
    assert_eq!(selected(&list), vec![1, 2, 4, 6]);
    for input in [ListBoxInput::Select(1), ListBoxInput::Activate(1)] {
        let update = list.apply(input);
        assert!(!update.changed && update.events.is_empty());
    }
    assert_eq!(list.drag_keys(&1), None);
    assert_eq!(list.drag_keys(&4), Some(vec![1, 2, 4, 6]));
    list.apply(ListBoxInput::ClearSelection);
    assert!(selected(&list).is_empty());
}

#[test]
fn projection_empty_and_disabled_views_keep_required_source_selection() {
    let mut list = list();
    list.set_selection_mode(SelectionMode::SingleRequired);
    for keys in [vec![], vec![3]] {
        list.set_projection(keys).unwrap();
        assert_eq!(list.active_key(), None);
        assert_eq!(selected(&list), vec![1]);
        assert!(!list.apply(ListBoxInput::Navigate(ListBoxNavigation::First)).changed);
        assert!(list.apply(ListBoxInput::ConfirmActive).events.is_empty());
    }
    list.replace_snapshot_with_projection(
        ListBoxSnapshot::try_with_enabled([1, 2, 3], |n| *n, |n| *n != 1).unwrap(),
        Some(vec![]),
    )
    .unwrap();
    assert_eq!(selected(&list), vec![2]);
    assert_eq!(list.active_key(), None);
    list.reset_projection();
    assert_eq!(list.active_key(), Some(&2));
}

#[test]
fn projection_replacements_compare_visible_keys_and_default_back_to_source_order() {
    let mut list = list();
    list.set_projection([4, 2]).unwrap();
    list.set_selected_keys([2, 6]).unwrap();
    let update = list
        .replace_snapshot_with_projection(ListBoxSnapshot::try_new([6, 2, 4, 1], |n| *n).unwrap(), Some(vec![4, 2]))
        .unwrap();
    assert!(update.changed);
    assert!(update.events.is_empty()); // source reordering isn't a view/selection membership change
    assert_eq!(selected(&list), vec![6, 2]);
    let update = list.replace_snapshot(ListBoxSnapshot::try_new([6, 2, 4, 1], |n| *n).unwrap());
    assert_eq!(visible(&list), vec![6, 2, 4, 1]);
    assert_eq!(update.events, vec![ListBoxEvent::ProjectionChanged { visible_count: 4 }]);
    list.set_projection([2]).unwrap();
    list.replace_snapshot_with_selection(ListBoxSnapshot::try_new([9, 8], |n| *n).unwrap(), [8], None)
        .unwrap();
    assert_eq!(visible(&list), vec![9, 8]);
    assert_eq!(list.active_key(), Some(&8));
}
