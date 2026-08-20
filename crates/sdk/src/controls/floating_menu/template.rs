use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, SharedString, Stateful, Window, div, px, relative, prelude::*, svg};
use lucide_svg_static::Icon as LucideIcon;

use crate::controls::choice_indicator_layout::shadow_extent_from_slice;
use crate::controls::menu_item::{MenuItem, MenuItemIcon};
use crate::controls::overlay_presence::OverlayPresence;
use crate::controls::state::MenuPath;
use crate::controls::floating_menu::FloatingMenuLook;
use crate::controls::icon::{DisclosureIcons, render_icon_source};

pub type FloatingMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type FloatingMenuHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug)]
pub struct FloatingMenuHighlight {
    pub from: MenuPath,
    pub to: MenuPath,
    pub progress: f32,
}

pub struct FloatingMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [MenuItem],
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub highlight: Option<FloatingMenuHighlight>,
    pub submenu_presence: OverlayPresence,
    pub look: FloatingMenuLook,
    pub disclosure_icons: &'a DisclosureIcons,
}

pub struct FloatingMenuTemplateHandlers {
    pub item_hovers: Vec<FloatingMenuHoverHandler>,
    pub submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    pub controlled_hover: bool,
    pub item_clicks: Vec<FloatingMenuClickHandler>,
}

pub type FloatingMenuTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait FloatingMenuTemplate: Send + Sync {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div>;
}

pub struct ThemedFloatingMenuTemplate {
    modifiers: Vec<FloatingMenuTemplateModifier>,
}

impl ThemedFloatingMenuTemplate {
    pub fn new() -> Self {
        Self { modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &FloatingMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl Default for ThemedFloatingMenuTemplate {
    fn default() -> Self {
        Self::new()
    }
}

struct ModifiedFloatingMenuTemplate {
    base: Arc<dyn FloatingMenuTemplate>,
    modifiers: Vec<FloatingMenuTemplateModifier>,
}

impl ModifiedFloatingMenuTemplate {
    fn new(base: Arc<dyn FloatingMenuTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: FloatingMenuTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &FloatingMenuRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl FloatingMenuTemplate for ThemedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let FloatingMenuTemplateHandlers { item_hovers, mut submenu_hovers, controlled_hover, item_clicks } = handlers;
        let look = model.look.clone();

        let mut menu = div()
            .id(format!("{}-menu", model.id))
            .relative()
            .min_w(px(look.min_width))
            .p(px(look.padding))
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow.clone())
            .occlude();

        if controlled_hover
            && let Some(highlight) = model.highlight
            && let Some((left, top, width, height)) = highlight_rect(highlight, false, &look)
        {
            menu = menu.child(
                div()
                    .absolute()
                    .left(px(left))
                    .top(px(top))
                    .w(px(width))
                    .h(px(height))
                    .rounded(px(look.item_radius))
                    .bg(look.item_hover_background),
            );
        }

        let mut item_clicks = item_clicks.into_iter();
        let mut submenu = None;

        for ((index, item), item_hover) in model.items.iter().enumerate().zip(item_hovers) {
            let enabled = item.is_enabled();
            let color = if enabled {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let mut row = div()
                .id(format!("{}-item-{}", model.id, item.id()))
                .flex()
                .items_center()
                .gap(px(look.item_gap))
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(render_item_icon(item.icon_ref(), look.item_icon_size))
                .child(div().flex_1().child(item.label_text().clone()));

            let is_active = matches!(model.active_path, Some(MenuPath::Root(active)) if active == index);

            if enabled {
                row = row.cursor_pointer().on_hover(item_hover).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    look.item_icon_size,
                    model.disclosure_icons,
                ));

                if !controlled_hover {
                    row = row.hover({
                        let hover_background = look.item_hover_background;
                        let hover_foreground = look.item_hover_foreground;
                        move |style| style.bg(hover_background).text_color(hover_foreground)
                    });
                }

                if is_active && !(controlled_hover && model.highlight.is_some()) {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if item.submenu_items().is_empty() {
                    if let Some(item_click) = item_clicks.next() {
                        row = row.on_click(item_click);
                    }
                } else if model.open_submenu == Some(index) {
                    submenu = Some(render_floating_submenu(
                        model.id,
                        item,
                        &look,
                        &mut item_clicks,
                        index,
                        model.active_path,
                        (index < submenu_hovers.len()).then(|| std::mem::take(&mut submenu_hovers[index])),
                        controlled_hover,
                        model.highlight,
                        model.submenu_presence.opacity(),
                        model.disclosure_icons,
                    ));
                }
            } else {
                row = row.opacity(look.disabled_opacity).child(render_submenu_affordance(
                    !item.submenu_items().is_empty(),
                    look.item_icon_size,
                    model.disclosure_icons,
                ));
            }

            menu = menu.child(row);
        }

        if let Some(submenu) = submenu {
            menu = menu.child(submenu);
        }

        let menu = with_elevation_slot(format!("{}-menu", model.id), menu, &look.shadow);
        self.apply_modifiers(menu, model)
    }
}

impl FloatingMenuTemplate for ModifiedFloatingMenuTemplate {
    fn render(&self, model: &FloatingMenuRenderModel<'_>, handlers: FloatingMenuTemplateHandlers) -> Stateful<Div> {
        let root = self.base.render(model, handlers);
        self.apply_modifiers(root, model)
    }
}

pub fn default_floating_menu_template() -> Arc<dyn FloatingMenuTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn FloatingMenuTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedFloatingMenuTemplate::new())).clone()
}

pub fn floating_menu_template_with_modifier<F>(
    template: Arc<dyn FloatingMenuTemplate>,
    modifier: F,
) -> Arc<dyn FloatingMenuTemplate>
where
    F: Fn(Stateful<Div>, &FloatingMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedFloatingMenuTemplate::new(template).with_modifier(Box::new(modifier)))
}

pub fn render_floating_menu(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    render_floating_menu_with_template(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_presence(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
) -> Stateful<Div> {
    render_floating_menu_with_template_and_submenu_presence(
        default_floating_menu_template(),
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        item_clicks,
        submenu_presence,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_template(
    template: Arc<dyn FloatingMenuTemplate>,
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> Stateful<Div> {
    let disclosure_icons = DisclosureIcons::default();
    template.render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence: OverlayPresence::new(true, false),
            look,
            disclosure_icons: &disclosure_icons,
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers: Vec::new(), controlled_hover: false, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
fn render_floating_menu_with_template_and_submenu_presence(
    template: Arc<dyn FloatingMenuTemplate>,
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    submenu_presence: OverlayPresence,
) -> Stateful<Div> {
    template.render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight: None,
            submenu_presence,
            look,
            disclosure_icons: &DisclosureIcons::default(),
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers: Vec::new(), controlled_hover: false, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_hovers(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
) -> Stateful<Div> {
    render_floating_menu_with_submenu_hovers_and_icons(
        id,
        items,
        open_submenu,
        active_path,
        look,
        item_hovers,
        submenu_hovers,
        item_clicks,
        highlight,
        DisclosureIcons::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_floating_menu_with_submenu_hovers_and_icons(
    id: &SharedString,
    items: &[MenuItem],
    open_submenu: Option<usize>,
    active_path: Option<MenuPath>,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    submenu_hovers: Vec<Vec<FloatingMenuHoverHandler>>,
    item_clicks: Vec<FloatingMenuClickHandler>,
    highlight: Option<FloatingMenuHighlight>,
    disclosure_icons: DisclosureIcons,
) -> Stateful<Div> {
    default_floating_menu_template().render(
        &FloatingMenuRenderModel {
            id,
            items,
            open_submenu,
            active_path,
            highlight,
            submenu_presence: OverlayPresence::new(true, false),
            look,
            disclosure_icons: &disclosure_icons,
        },
        FloatingMenuTemplateHandlers { item_hovers, submenu_hovers, controlled_hover: true, item_clicks },
    )
}

#[allow(clippy::too_many_arguments)]
fn render_floating_submenu(
    menu_id: &SharedString,
    item: &MenuItem,
    look: &FloatingMenuLook,
    item_clicks: &mut std::vec::IntoIter<FloatingMenuClickHandler>,
    index: usize,
    active_path: Option<MenuPath>,
    submenu_hovers: Option<Vec<FloatingMenuHoverHandler>>,
    controlled_hover: bool,
    highlight: Option<FloatingMenuHighlight>,
    submenu_opacity: f32,
    disclosure_icons: &DisclosureIcons,
) -> Stateful<Div> {
    let mut submenu = div()
        .id(format!("{menu_id}-submenu-{}", item.id()))
        .absolute()
        .top(px(look.padding + (index as f32 * look.item_height)))
        .left(relative(1.0))
        .ml(px(0.0))
        .min_w(px(look.min_width))
        .p(px(look.padding))
        .bg(look.background)
        .border_1()
        .border_color(look.border)
        .rounded(px(look.radius))
        .shadow(look.shadow.clone())
        .occlude();

    if controlled_hover
        && let Some(highlight) = highlight
        && let Some((left, top, width, height)) = highlight_rect(highlight, true, look)
    {
        submenu = submenu.child(
            div()
                .absolute()
                .left(px(left))
                .top(px(top))
                .w(px(width))
                .h(px(height))
                .rounded(px(look.item_radius))
                .bg(look.item_hover_background),
        );
    }

    let mut submenu_hovers = submenu_hovers.map(Vec::into_iter);
    for (submenu_index, submenu_item) in item.submenu_items().iter().enumerate() {
        let enabled = submenu_item.is_enabled();
        let color = if enabled {
            look.foreground
        } else {
            look.item_disabled_foreground
        };
        let mut row = div()
            .id(format!("{menu_id}-submenu-item-{}", submenu_item.id()))
            .flex()
            .items_center()
            .gap(px(look.item_gap))
            .min_h(px(look.item_height))
            .px(px(look.item_padding_x))
            .rounded(px(look.item_radius))
            .text_color(color)
            .text_size(px(look.item_typography.size))
            .line_height(px(look.item_typography.line_height))
            .font_weight(look.item_typography.weight)
            .child(render_item_icon(submenu_item.icon_ref(), look.item_icon_size))
            .child(div().flex_1().child(submenu_item.label_text().clone()))
            .child(render_submenu_affordance(
                !submenu_item.submenu_items().is_empty(),
                look.item_icon_size,
                disclosure_icons,
            ));

        if enabled && submenu_item.submenu_items().is_empty() {
            if let Some(hover) = submenu_hovers.as_mut().and_then(Iterator::next) {
                row = row.on_hover(hover);
            }
            if let Some(item_click) = item_clicks.next() {
                row = row.cursor_pointer().on_click(item_click);
                if !controlled_hover {
                    row = row.hover({
                        let hover_background = look.item_hover_background;
                        let hover_foreground = look.item_hover_foreground;
                        move |style| style.bg(hover_background).text_color(hover_foreground)
                    });
                }
            }

            if active_path.is_some_and(|path| path.is_submenu(index, submenu_index))
                && !(controlled_hover && highlight.is_some())
            {
                row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
            }
        } else if !enabled {
            row = row.opacity(look.disabled_opacity);
        }

        submenu = submenu.child(row);
    }

    submenu.opacity(submenu_opacity)
}

fn highlight_rect(
    highlight: FloatingMenuHighlight,
    submenu: bool,
    look: &FloatingMenuLook,
) -> Option<(f32, f32, f32, f32)> {
    let index = match (submenu, highlight.from) {
        (false, MenuPath::Root(index)) => Some(index),
        (true, MenuPath::Submenu { child, .. }) => Some(child),
        _ => None,
    }? as f32;
    let to = match (submenu, highlight.to) {
        (false, MenuPath::Root(index)) => index as f32,
        (true, MenuPath::Submenu { child, .. }) => child as f32,
        _ => return None,
    };
    let row = index + ((to - index) * highlight.progress.clamp(0.0, 1.0));
    Some((
        look.padding,
        look.padding + row * look.item_height,
        (look.min_width - look.padding * 2.0).max(0.0),
        look.item_height,
    ))
}

fn with_elevation_slot(
    id: impl Into<SharedString>,
    surface: Stateful<Div>,
    shadows: &[gpui::BoxShadow],
) -> Stateful<Div> {
    let extent = shadow_extent_from_slice(shadows, 1.0, true);
    if extent <= 0.0 {
        return surface;
    }
    div().id(format!("{}-elevation", id.into())).relative().p(px(extent)).child(surface)
}

fn render_item_icon(icon: Option<&MenuItemIcon>, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, size)
    } else if let Some(path) = icon.and_then(MenuItemIcon::svg_path) {
        svg().external_path(path.clone()).size(px(size)).into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(has_submenu: bool, size: f32, icons: &DisclosureIcons) -> AnyElement {
    if has_submenu {
        render_icon_source(&icons.collapsed, gpui::Hsla::default(), size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_lucide_icon(icon: LucideIcon, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .line_height(px(size))
        .child(crate::controls::icon::lucide_icon(icon, gpui::hsla(0.0, 0.0, 1.0, 1.0), size))
        .into_any_element()
}
