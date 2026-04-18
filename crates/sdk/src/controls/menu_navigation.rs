use gpui::KeyDownEvent;

use crate::controls::dropdown_menu::DropdownMenuItem;
use crate::controls::state::MenuPath;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MenuDirection {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MenuKey {
    OpenFirst,
    OpenLast,
    Previous,
    Next,
    First,
    Last,
    OpenSubmenu,
    CloseSubmenu,
    Select,
}

impl MenuKey {
    pub(crate) fn from_key_down(event: &KeyDownEvent) -> Option<Self> {
        if event.keystroke.modifiers.modified() {
            return None;
        }

        match event.keystroke.key.as_str() {
            "down" => Some(Self::Next),
            "up" => Some(Self::Previous),
            "home" => Some(Self::First),
            "end" => Some(Self::Last),
            "right" => Some(Self::OpenSubmenu),
            "left" => Some(Self::CloseSubmenu),
            "enter" | "space" => Some(Self::Select),
            _ => None,
        }
    }

    pub(crate) fn opening_key(event: &KeyDownEvent) -> Option<Self> {
        if event.keystroke.modifiers.modified() {
            return None;
        }

        match event.keystroke.key.as_str() {
            "down" | "enter" | "space" => Some(Self::OpenFirst),
            "up" => Some(Self::OpenLast),
            _ => None,
        }
    }
}

pub(crate) struct MenuNavigator<'a> {
    items: &'a [DropdownMenuItem],
}

impl<'a> MenuNavigator<'a> {
    pub(crate) fn new(items: &'a [DropdownMenuItem]) -> Self {
        Self { items }
    }

    pub(crate) fn first_root(&self) -> Option<usize> {
        self.step_root(None, MenuDirection::Next)
    }

    pub(crate) fn last_root(&self) -> Option<usize> {
        self.step_root(None, MenuDirection::Previous)
    }

    pub(crate) fn step_root(
        &self,
        current: Option<usize>,
        direction: MenuDirection,
    ) -> Option<usize> {
        step_enabled_index(self.items.len(), current, direction, |index| {
            self.items[index].is_enabled()
        })
    }

    pub(crate) fn first_submenu(&self, parent: usize) -> Option<usize> {
        self.step_submenu(parent, None, MenuDirection::Next)
    }

    pub(crate) fn last_submenu(&self, parent: usize) -> Option<usize> {
        self.step_submenu(parent, None, MenuDirection::Previous)
    }

    pub(crate) fn step_submenu(
        &self,
        parent: usize,
        current: Option<usize>,
        direction: MenuDirection,
    ) -> Option<usize> {
        let submenu_items = self.items.get(parent)?.submenu_items();

        step_enabled_index(submenu_items.len(), current, direction, |index| {
            submenu_items[index].is_enabled()
        })
    }

    pub(crate) fn active_root(&self, active_path: Option<MenuPath>) -> Option<usize> {
        match active_path {
            Some(MenuPath::Root(index)) => Some(index),
            Some(MenuPath::Submenu { parent, .. }) => Some(parent),
            None => None,
        }
        .filter(|index| {
            self.items
                .get(*index)
                .is_some_and(DropdownMenuItem::is_enabled)
        })
    }

    pub(crate) fn active_item(
        &self,
        active_path: Option<MenuPath>,
    ) -> Option<&'a DropdownMenuItem> {
        match active_path {
            Some(MenuPath::Root(index)) => self.items.get(index),
            Some(MenuPath::Submenu { parent, child }) => {
                self.items.get(parent)?.submenu_items().get(child)
            }
            None => None,
        }
    }
}

fn step_enabled_index(
    len: usize,
    current: Option<usize>,
    direction: MenuDirection,
    is_enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let step = match direction {
        MenuDirection::Previous => len - 1,
        MenuDirection::Next => 1,
    };
    let mut index = match current.filter(|index| *index < len) {
        Some(current) => (current + step) % len,
        None if direction == MenuDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if is_enabled(index) {
            return Some(index);
        }

        index = (index + step) % len;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::dropdown_menu::DropdownMenuItem;

    #[test]
    fn root_navigation_skips_disabled_items_and_wraps() {
        let items = vec![
            DropdownMenuItem::new("first"),
            DropdownMenuItem::new("disabled").enabled(false),
            DropdownMenuItem::new("last"),
        ];
        let navigator = MenuNavigator::new(&items);

        assert_eq!(navigator.first_root(), Some(0));
        assert_eq!(navigator.last_root(), Some(2));
        assert_eq!(navigator.step_root(Some(0), MenuDirection::Next), Some(2));
        assert_eq!(navigator.step_root(Some(2), MenuDirection::Next), Some(0));
        assert_eq!(
            navigator.step_root(Some(0), MenuDirection::Previous),
            Some(2)
        );
    }

    #[test]
    fn submenu_navigation_skips_disabled_items_and_wraps() {
        let items = vec![DropdownMenuItem::new("parent").submenu([
            DropdownMenuItem::new("first"),
            DropdownMenuItem::new("disabled").enabled(false),
            DropdownMenuItem::new("last"),
        ])];
        let navigator = MenuNavigator::new(&items);

        assert_eq!(navigator.first_submenu(0), Some(0));
        assert_eq!(navigator.last_submenu(0), Some(2));
        assert_eq!(
            navigator.step_submenu(0, Some(0), MenuDirection::Next),
            Some(2)
        );
        assert_eq!(
            navigator.step_submenu(0, Some(2), MenuDirection::Next),
            Some(0)
        );
    }
}
