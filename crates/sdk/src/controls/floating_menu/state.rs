use gpui::SharedString;

use crate::controls::menu_item::MenuItem;
use crate::controls::menu_navigation::{MenuDirection, MenuNavigator};
use crate::controls::state::MenuPath;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FloatingMenuState {
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingMenuStepDirection {
    Previous,
    Next,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FloatingMenuActivateResult {
    None,
    OpenedSubmenu,
    Select { item_id: SharedString, label: SharedString },
}

impl FloatingMenuState {
    pub fn open_submenu(&self) -> Option<usize> {
        self.open_submenu
    }

    pub fn active_path(&self) -> Option<MenuPath> {
        self.active_path
    }

    pub fn clear(&mut self) {
        self.open_submenu = None;
        self.active_path = None;
    }

    pub fn open_with(&mut self, active_path: Option<MenuPath>) -> bool {
        let open_submenu = match active_path {
            Some(MenuPath::Submenu { parent, .. }) => Some(parent),
            _ => None,
        };

        let changed = self.open_submenu != open_submenu || self.active_path != active_path;
        self.open_submenu = open_submenu;
        self.active_path = active_path;
        changed
    }

    pub fn item_click_paths(&self, items: &[MenuItem]) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();

        for (index, item) in items.iter().enumerate() {
            if item.enabled && item.submenu_items.is_empty() {
                paths.push(vec![index]);
            } else if self.open_submenu == Some(index) {
                paths.extend(
                    item.submenu_items
                        .iter()
                        .enumerate()
                        .filter(|(_, submenu_item)| submenu_item.enabled && submenu_item.submenu_items.is_empty())
                        .map(|(submenu_index, _)| vec![index, submenu_index]),
                );
            }
        }

        paths
    }

    pub fn hover_root_item(&mut self, items: &[MenuItem], index: usize) -> bool {
        let next_submenu =
            items.get(index).is_some_and(|item| item.enabled && !item.submenu_items.is_empty()).then_some(index);

        let next_active_path = Some(MenuPath::Root(index));
        let changed = self.open_submenu != next_submenu || self.active_path != next_active_path;
        if changed {
            self.open_submenu = next_submenu;
            self.active_path = next_active_path;
        }
        changed
    }

    pub fn hover_submenu_item(&mut self, items: &[MenuItem], parent: usize, child: usize) -> bool {
        let Some(item) = items.get(parent).and_then(|item| item.submenu_items().get(child)) else {
            return false;
        };
        if !item.is_enabled() || !item.submenu_items().is_empty() {
            return false;
        }

        let next_path = Some(MenuPath::Submenu { parent, child });
        let changed = self.open_submenu != Some(parent) || self.active_path != next_path;
        if changed {
            self.open_submenu = Some(parent);
            self.active_path = next_path;
        }
        changed
    }

    pub fn select_at_path(&self, items: &[MenuItem], path: &[usize]) -> Option<(SharedString, SharedString)> {
        let item = match path {
            [index] => items.get(*index),
            [index, submenu_index] => items.get(*index)?.submenu_items.get(*submenu_index),
            _ => None,
        }?;

        if !item.enabled || !item.submenu_items.is_empty() {
            return None;
        }

        Some((item.id.clone(), item.label.clone()))
    }

    pub fn step(&mut self, items: &[MenuItem], direction: FloatingMenuStepDirection) -> bool {
        let direction = match direction {
            FloatingMenuStepDirection::Previous => MenuDirection::Previous,
            FloatingMenuStepDirection::Next => MenuDirection::Next,
        };

        let navigator = MenuNavigator::new(items);
        let next_path = match self.active_path {
            Some(MenuPath::Submenu { parent, child }) => navigator
                .step_submenu(parent, Some(child), direction)
                .map(|child| MenuPath::Submenu { parent, child }),
            _ => navigator.step_root(navigator.active_root(self.active_path), direction).map(MenuPath::Root),
        };

        if let Some(next_path) = next_path
            && self.active_path != Some(next_path)
        {
            self.active_path = Some(next_path);
            if matches!(next_path, MenuPath::Root(_)) {
                self.open_submenu = None;
            }
            return true;
        }

        false
    }

    pub fn move_to_boundary(&mut self, items: &[MenuItem], first: bool) -> bool {
        let navigator = MenuNavigator::new(items);
        let next_path = match self.active_path {
            Some(MenuPath::Submenu { parent, .. }) => {
                let child = if first {
                    navigator.first_submenu(parent)
                } else {
                    navigator.last_submenu(parent)
                };

                child.map(|child| MenuPath::Submenu { parent, child })
            }
            _ => {
                let root = if first {
                    navigator.first_root()
                } else {
                    navigator.last_root()
                };

                root.map(MenuPath::Root)
            }
        };

        if let Some(next_path) = next_path
            && self.active_path != Some(next_path)
        {
            self.active_path = Some(next_path);
            if matches!(next_path, MenuPath::Root(_)) {
                self.open_submenu = None;
            }
            return true;
        }

        false
    }

    pub fn open_active_submenu(&mut self, items: &[MenuItem]) -> bool {
        let navigator = MenuNavigator::new(items);
        if let Some(parent) = navigator.active_root(self.active_path)
            && let Some(child) = navigator.first_submenu(parent)
        {
            let next_path = Some(MenuPath::Submenu { parent, child });
            if self.open_submenu != Some(parent) || self.active_path != next_path {
                self.open_submenu = Some(parent);
                self.active_path = next_path;
                return true;
            }
        }

        false
    }

    pub fn close_active_submenu(&mut self) -> bool {
        if let Some(MenuPath::Submenu { parent, .. }) = self.active_path {
            self.open_submenu = None;
            self.active_path = Some(MenuPath::Root(parent));
            return true;
        }

        false
    }

    pub fn activate(&mut self, items: &[MenuItem]) -> FloatingMenuActivateResult {
        let navigator = MenuNavigator::new(items);
        let Some(active_path) = self.active_path else {
            return FloatingMenuActivateResult::None;
        };

        let Some(item) = navigator.active_item(Some(active_path)) else {
            return FloatingMenuActivateResult::None;
        };

        if !item.submenu_items().is_empty() {
            if let MenuPath::Root(parent) = active_path
                && let Some(child) = navigator.first_submenu(parent)
            {
                self.open_submenu = Some(parent);
                self.active_path = Some(MenuPath::Submenu { parent, child });
                return FloatingMenuActivateResult::OpenedSubmenu;
            }
            return FloatingMenuActivateResult::None;
        }

        FloatingMenuActivateResult::Select { item_id: item.id.clone(), label: item.label.clone() }
    }
}
