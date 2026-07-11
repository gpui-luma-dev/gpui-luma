use gpui::SharedString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::gallery) enum HeadlessGroupDirection {
    Previous,
    Next,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HeadlessGroupItem {
    id: SharedString,
    enabled: bool,
}

pub(in crate::gallery) struct HeadlessControlGroup {
    items: Vec<HeadlessGroupItem>,
    active_id: Option<SharedString>,
    selected_id: Option<SharedString>,
    activation_follows_focus: bool,
}

impl HeadlessControlGroup {
    pub(in crate::gallery) fn new(
        items: impl IntoIterator<Item = (impl Into<SharedString>, bool)>,
        selected_id: Option<impl Into<SharedString>>,
        activation_follows_focus: bool,
    ) -> Self {
        let items = items
            .into_iter()
            .map(|(id, enabled)| HeadlessGroupItem { id: id.into(), enabled })
            .collect::<Vec<_>>();
        let selected_id = selected_id
            .map(Into::into)
            .filter(|selected| items.iter().any(|item| item.enabled && &item.id == selected));
        let active_id = selected_id.clone().or_else(|| first_enabled_id(&items));

        Self { items, active_id, selected_id, activation_follows_focus }
    }

    #[allow(dead_code)]
    pub(in crate::gallery) fn active_id(&self) -> Option<&SharedString> {
        self.active_id.as_ref()
    }

    pub(in crate::gallery) fn selected_id(&self) -> Option<&SharedString> {
        self.selected_id.as_ref()
    }

    pub(in crate::gallery) fn move_active(&mut self, direction: HeadlessGroupDirection) -> bool {
        if self.items.is_empty() || self.items.iter().all(|item| !item.enabled) {
            return false;
        }

        let current_index = self
            .active_id
            .as_ref()
            .and_then(|active_id| self.items.iter().position(|item| item.enabled && &item.id == active_id))
            .unwrap_or(0);
        let len = self.items.len();
        let step = match direction {
            HeadlessGroupDirection::Previous => len - 1,
            HeadlessGroupDirection::Next => 1,
        };
        let mut next_index = current_index;

        for _ in 0..len {
            next_index = (next_index + step) % len;
            if self.items[next_index].enabled {
                return self.set_active_index(next_index);
            }
        }

        false
    }

    pub(in crate::gallery) fn move_active_to_boundary(&mut self, first: bool) -> bool {
        let boundary_index = if first {
            self.items.iter().position(|item| item.enabled)
        } else {
            self.items.iter().rposition(|item| item.enabled)
        };

        let Some(boundary_index) = boundary_index else {
            return false;
        };

        self.set_active_index(boundary_index)
    }

    pub(in crate::gallery) fn activate_active(&mut self) -> bool {
        let Some(active_id) = self.active_id.clone() else {
            return false;
        };

        self.select(active_id)
    }

    pub(in crate::gallery) fn select(&mut self, selected_id: impl Into<SharedString>) -> bool {
        let selected_id = selected_id.into();
        if !self.items.iter().any(|item| item.enabled && item.id == selected_id) {
            return false;
        }

        let selection_changed = self.selected_id.as_ref() != Some(&selected_id);
        let active_changed = self.active_id.as_ref() != Some(&selected_id);

        self.selected_id = Some(selected_id.clone());
        self.active_id = Some(selected_id);
        selection_changed || active_changed
    }

    fn set_active_index(&mut self, index: usize) -> bool {
        if !self.items[index].enabled {
            return false;
        }

        let next_active = self.items[index].id.clone();
        let active_changed = self.active_id.as_ref() != Some(&next_active);
        let selection_changed = self.activation_follows_focus && self.selected_id.as_ref() != Some(&next_active);

        if !active_changed && !selection_changed {
            return false;
        }

        self.active_id = Some(next_active.clone());
        if self.activation_follows_focus {
            self.selected_id = Some(next_active);
        }

        true
    }
}

fn first_enabled_id(items: &[HeadlessGroupItem]) -> Option<SharedString> {
    items.iter().find(|item| item.enabled).map(|item| item.id.clone())
}
