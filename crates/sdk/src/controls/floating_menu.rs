use gpui::{AnyElement, App, ClickEvent, Div, FontWeight, SharedString, Stateful, Window, div, px, prelude::*, svg};
use lucide_icons::Icon as LucideIcon;

use crate::controls::menu_item::{MenuItem, MenuItemIcon};
use crate::controls::menu_navigation::{MenuDirection, MenuNavigator};
use crate::controls::state::MenuPath;
use crate::theme::FloatingMenuAppearance;

pub type FloatingMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type FloatingMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

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

pub fn render_floating_menu(
    id: &gpui::SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    appearance: FloatingMenuAppearance,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    let mut menu = div()
        .id(format!("{id}-menu"))
        .relative()
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .occlude();
    let mut item_clicks = item_clicks.into_iter();
    let mut submenu = None;

    for ((index, item), item_hover) in items.iter().enumerate().zip(item_hovers) {
        let enabled = item.is_enabled();
        let color = if enabled {
            appearance.foreground
        } else {
            appearance.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{id}-item-{}", item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(color)
            .text_size(px(appearance.item_typography.size))
            .line_height(px(appearance.item_typography.line_height))
            .font_weight(appearance.item_typography.weight)
            .child(render_item_icon(item.icon_ref(), color, appearance.item_icon_size))
            .child(div().flex_1().child(item.label_text().clone()));

        if enabled {
            row = row
                .cursor_pointer()
                .on_hover(item_hover)
                .hover({
                    let hover_background = appearance.item_hover_background;
                    move |style| style.bg(hover_background)
                })
                .child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    appearance.foreground,
                    appearance.item_icon_size,
                ));

            if active_path.is_some_and(|active_path| active_path.is_root(index)) {
                row = row.bg(appearance.item_hover_background);
            }

            if item.submenu_items().is_empty() {
                if let Some(item_click) = item_clicks.next() {
                    row = row.on_click(item_click);
                }
            } else if open_submenu == Some(index) {
                submenu = Some(render_floating_submenu(id, item, &appearance, &mut item_clicks, index, active_path));
            }
        } else {
            row = row.opacity(0.56).child(render_submenu_affordance(
                !item.submenu_items().is_empty(),
                appearance.item_disabled_foreground,
                appearance.item_icon_size,
            ));
        }

        menu = menu.child(row);
    }

    if let Some(submenu) = submenu {
        menu = menu.child(submenu);
    }

    menu
}

fn render_floating_submenu(
    menu_id: &gpui::SharedString,
    item: &MenuItem,
    appearance: &FloatingMenuAppearance,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{menu_id}-submenu-{}", item.id()))
        .absolute()
        .top(px(appearance.padding + (index as f32 * appearance.item_height)))
        .left(px(appearance.min_width + appearance.submenu_offset_x))
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .occlude();

    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let enabled = submenu_item.is_enabled();
        let color = if enabled {
            appearance.foreground
        } else {
            appearance.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{menu_id}-submenu-item-{}", submenu_item.id()))
            .flex()
            .items_center()
            .gap(px(appearance.item_gap))
            .min_h(px(appearance.item_height))
            .px(px(appearance.item_padding_x))
            .rounded(px(appearance.item_radius))
            .text_color(color)
            .text_size(px(appearance.item_typography.size))
            .line_height(px(appearance.item_typography.line_height))
            .font_weight(appearance.item_typography.weight)
            .child(render_item_icon(submenu_item.icon_ref(), color, appearance.item_icon_size))
            .child(div().flex_1().child(submenu_item.label_text().clone()));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(item_click) = item_clicks.next() {
                row = row
                    .cursor_pointer()
                    .hover({
                        let hover_background = appearance.item_hover_background;
                        move |style| style.bg(hover_background)
                    })
                    .on_click(item_click);
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index)) {
                row = row.bg(appearance.item_hover_background);
            }
        } else if !enabled {
            row = row.opacity(0.56);
        }

        submenu = submenu.child(row);
    }

    submenu
}

fn render_item_icon(icon: Option<&MenuItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).text_color(color).into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(has_submenu: bool, color: gpui::Hsla, size: f32) -> AnyElement {
    if has_submenu {
        render_lucide_icon(LucideIcon::ChevronRight, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
