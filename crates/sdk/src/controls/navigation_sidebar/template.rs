use std::sync::{Arc, OnceLock};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Corner, Div, FocusHandle, FontFeatures, FontWeight, MouseButton,
    MouseDownEvent, MouseUpEvent, Pixels, SharedString, Stateful, Window, anchored, deferred, div, point, prelude::*,
    px,
};
use lucide_icons::Icon as LucideIcon;

use super::{NavNodeKind, NavigationSidebarRenderModel, RenderedCollapseTrigger, RenderedNavNode, RenderedRailSubmenu};
use crate::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler, render_floating_menu};
use crate::controls::scroll_container::ScrollContainer;
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, LumaTypography};
use crate::controls::floating_menu::{FloatingMenuLook, FloatingMenuTheme, default_floating_menu_theme};
use crate::controls::navigation_sidebar::{NavigationSidebarTheme, default_navigation_sidebar_theme};

const CONTAINER_GAP: f32 = 8.0;
const CONTAINER_PADDING: f32 = 8.0;
const REGION_GAP: f32 = 4.0;
const HEADER_REGION_PADDING_BOTTOM: f32 = 8.0;
const FOOTER_REGION_PADDING_TOP: f32 = 8.0;
const TITLE_GAP: f32 = 2.0;
const TITLE_PADDING_BOTTOM: f32 = 4.0;
const SUBTITLE_OPACITY: f32 = 0.72;
const SECTION_PADDING_TOP: f32 = 8.0;
const CHILD_DEPTH_INDENT_MULTIPLIER: f32 = 1.0;
const DISCLOSURE_EXPANDED_ICON: LucideIcon = LucideIcon::ChevronDown;
const DISCLOSURE_COLLAPSED_ICON: LucideIcon = LucideIcon::ChevronRight;
const DISCLOSURE_ICON_SIZE: f32 = 14.0;
const COLLAPSE_EXPANDED_ICON: LucideIcon = LucideIcon::PanelLeftClose;
const COLLAPSE_COLLAPSED_ICON: LucideIcon = LucideIcon::PanelLeftOpen;
const DISABLED_ROW_OPACITY: f32 = 0.56;
const SCROLL_REGION_MIN_HEIGHT: f32 = 0.0;
const RAIL_SUBMENU_OFFSET_X: f32 = 12.0;
const RAIL_BRANCH_INDICATOR_ICON: LucideIcon = LucideIcon::ChevronRight;
const RAIL_BRANCH_INDICATOR_SIZE: f32 = 18.0;
const RAIL_BRANCH_INDICATOR_RIGHT: f32 = -10.0;
const LUCIDE_FONT_FAMILY: &str = "lucide";
const ICON_FONT_WEIGHT: FontWeight = FontWeight::NORMAL;

pub type NavigationSidebarBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type NavigationSidebarClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type NavigationSidebarHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type NavigationSidebarMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type NavigationSidebarMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(Default)]
pub struct NavigationSidebarTemplateHandlers {
    pub collapse_hover: Option<NavigationSidebarHoverHandler>,
    pub collapse_mouse_down: Option<NavigationSidebarMouseDownHandler>,
    pub collapse_mouse_up: Option<NavigationSidebarMouseUpHandler>,
    pub collapse_mouse_up_out: Option<NavigationSidebarMouseUpHandler>,
    pub collapse_click: Option<NavigationSidebarClickHandler>,
    pub row_bounds: Vec<NavigationSidebarBoundsHandler>,
    pub row_hovers: Vec<NavigationSidebarHoverHandler>,
    pub row_mouse_downs: Vec<NavigationSidebarMouseDownHandler>,
    pub row_mouse_ups: Vec<NavigationSidebarMouseUpHandler>,
    pub row_mouse_up_outs: Vec<NavigationSidebarMouseUpHandler>,
    pub row_clicks: Vec<NavigationSidebarClickHandler>,
    pub rail_submenu_mouse_down_out: Option<NavigationSidebarMouseDownHandler>,
    pub rail_submenu_item_hovers: Vec<FloatingMenuHoverHandler>,
    pub rail_submenu_item_clicks: Vec<FloatingMenuClickHandler>,
}

pub type NavigationSidebarTemplateModifier = Box<dyn Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait NavigationSidebarTemplate: Send + Sync {
    fn render(
        &self,
        model: NavigationSidebarRenderModel,
        main_scroll: &ScrollContainer,
        handlers: NavigationSidebarTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedNavigationSidebarTemplate {
    theme: Arc<dyn NavigationSidebarTheme>,
    floating_menu_theme: Arc<dyn FloatingMenuTheme>,
    modifiers: Vec<NavigationSidebarTemplateModifier>,
}

impl ThemedNavigationSidebarTemplate {
    pub fn new(theme: Arc<dyn NavigationSidebarTheme>) -> Self {
        Self { theme, floating_menu_theme: default_floating_menu_theme(), modifiers: Vec::new() }
    }

    pub fn new_with_floating_menu_theme(
        theme: Arc<dyn NavigationSidebarTheme>,
        floating_menu_theme: Arc<dyn FloatingMenuTheme>,
    ) -> Self {
        Self { theme, floating_menu_theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root);
        }
        root
    }
}

struct ModifiedNavigationSidebarTemplate {
    base: Arc<dyn NavigationSidebarTemplate>,
    modifiers: Vec<NavigationSidebarTemplateModifier>,
}

impl ModifiedNavigationSidebarTemplate {
    fn new(base: Arc<dyn NavigationSidebarTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: NavigationSidebarTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root);
        }
        root
    }
}

impl Default for ThemedNavigationSidebarTemplate {
    fn default() -> Self {
        Self::new(default_navigation_sidebar_theme())
    }
}

pub fn default_navigation_sidebar_template() -> Arc<dyn NavigationSidebarTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn NavigationSidebarTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedNavigationSidebarTemplate::default())).clone()
}

pub(super) fn modified_navigation_sidebar_template<F>(
    template: Arc<dyn NavigationSidebarTemplate>,
    modifier: F,
) -> Arc<dyn NavigationSidebarTemplate>
where
    F: Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedNavigationSidebarTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl NavigationSidebarTemplate for ModifiedNavigationSidebarTemplate {
    fn render(
        &self,
        model: NavigationSidebarRenderModel,
        main_scroll: &ScrollContainer,
        handlers: NavigationSidebarTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, main_scroll, handlers, window, cx);
        self.apply_modifiers(root)
    }
}

impl NavigationSidebarTemplate for ThemedNavigationSidebarTemplate {
    fn render(
        &self,
        model: NavigationSidebarRenderModel,
        main_scroll: &ScrollContainer,
        handlers: NavigationSidebarTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let container = self.theme.resolve_container();
        let NavigationSidebarTemplateHandlers {
            collapse_hover,
            collapse_mouse_down,
            collapse_mouse_up,
            collapse_mouse_up_out,
            collapse_click,
            mut row_bounds,
            mut row_hovers,
            mut row_mouse_downs,
            mut row_mouse_ups,
            mut row_mouse_up_outs,
            mut row_clicks,
            rail_submenu_mouse_down_out,
            rail_submenu_item_hovers,
            rail_submenu_item_clicks,
        } = handlers;
        let mut row_bounds = row_bounds.drain(..);
        let mut row_hovers = row_hovers.drain(..);
        let mut row_mouse_downs = row_mouse_downs.drain(..);
        let mut row_mouse_ups = row_mouse_ups.drain(..);
        let mut row_mouse_up_outs = row_mouse_up_outs.drain(..);
        let mut row_clicks = row_clicks.drain(..);
        let mut root = div()
            .id(model.id)
            .size_full()
            .flex()
            .flex_col()
            .gap(px(CONTAINER_GAP))
            .p(px(CONTAINER_PADDING))
            .bg(container.background)
            .text_color(container.foreground);

        if model.collapsed {
            if let Some(trigger) = model.collapse_trigger {
                root = root.child(div().flex().justify_center().child(render_collapse_trigger(
                    trigger,
                    RowHandlers {
                        bounds: None,
                        hover: collapse_hover,
                        mouse_down: collapse_mouse_down,
                        mouse_up: collapse_mouse_up,
                        mouse_up_out: collapse_mouse_up_out,
                        click: collapse_click,
                    },
                    &self.theme,
                )));
            }

            root = root.child(
                render_collapsed_rail_region(
                    model.rail_nodes,
                    &self.theme,
                    &mut row_bounds,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                )
                .flex_1(),
            );

            if !model.rail_footer_nodes.is_empty() {
                root = root.child(
                    render_collapsed_rail_region(
                        model.rail_footer_nodes,
                        &self.theme,
                        &mut row_bounds,
                        &mut row_hovers,
                        &mut row_mouse_downs,
                        &mut row_mouse_ups,
                        &mut row_mouse_up_outs,
                        &mut row_clicks,
                    )
                    .pt(px(FOOTER_REGION_PADDING_TOP)),
                );
            }

            if let Some(rail_submenu) = model.rail_submenu {
                root = root
                    .on_mouse_down_out(
                        rail_submenu_mouse_down_out.expect("rail submenu should have outside click handler"),
                    )
                    .child(
                        deferred(render_rail_submenu_overlay(
                            rail_submenu,
                            self.floating_menu_theme.resolve(),
                            rail_submenu_item_hovers,
                            rail_submenu_item_clicks,
                        ))
                        .with_priority(1),
                    );
            }

            return root;
        }

        if model.title.is_some() || model.subtitle.is_some() || model.collapse_trigger.is_some() {
            root = root.child(render_title(
                model.title,
                model.subtitle,
                model.collapse_trigger,
                RowHandlers {
                    bounds: None,
                    hover: collapse_hover,
                    mouse_down: collapse_mouse_down,
                    mouse_up: collapse_mouse_up,
                    mouse_up_out: collapse_mouse_up_out,
                    click: collapse_click,
                },
                &self.theme,
            ));
        }

        if !model.header_nodes.is_empty() {
            root = root.child(
                render_region(
                    model.header_nodes,
                    &self.theme,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                )
                .pb(px(HEADER_REGION_PADDING_BOTTOM)),
            );
        }

        root = root.child(
            main_scroll
                .render(
                    render_region(
                        model.nodes,
                        &self.theme,
                        &mut row_hovers,
                        &mut row_mouse_downs,
                        &mut row_mouse_ups,
                        &mut row_mouse_up_outs,
                        &mut row_clicks,
                    )
                    .into_any_element(),
                )
                .flex_1()
                .min_h(px(SCROLL_REGION_MIN_HEIGHT)),
        );

        if !model.footer_nodes.is_empty() {
            root = root.child(
                render_region(
                    model.footer_nodes,
                    &self.theme,
                    &mut row_hovers,
                    &mut row_mouse_downs,
                    &mut row_mouse_ups,
                    &mut row_mouse_up_outs,
                    &mut row_clicks,
                )
                .pt(px(FOOTER_REGION_PADDING_TOP)),
            );
        }

        self.apply_modifiers(root)
    }
}

fn sidebar_title_style() -> LumaTextStyle {
    LumaTypography::default().text.label
}

fn sidebar_subtitle_style() -> LumaTextStyle {
    LumaTypography::default().text.scale.sm
}

fn render_title(
    title: Option<SharedString>,
    subtitle: Option<SharedString>,
    collapse_trigger: Option<RenderedCollapseTrigger>,
    collapse_handlers: RowHandlers,
    theme: &Arc<dyn NavigationSidebarTheme>,
) -> Div {
    let title_style = sidebar_title_style();
    let subtitle_style = sidebar_subtitle_style();
    let mut header = div().flex().flex_col().gap(px(TITLE_GAP)).pb(px(TITLE_PADDING_BOTTOM));
    let mut title_row = div().flex().items_center().gap(px(TITLE_GAP));

    if let Some(title) = title {
        title_row = title_row.child(
            div()
                .flex_1()
                .text_size(px(title_style.size))
                .line_height(px(title_style.line_height))
                .font_weight(title_style.weight)
                .child(title),
        );
    } else {
        title_row = title_row.child(div().flex_1());
    }

    if let Some(trigger) = collapse_trigger {
        title_row = title_row.child(render_collapse_trigger(trigger, collapse_handlers, theme));
    }

    header = header.child(title_row);

    if let Some(subtitle) = subtitle {
        header = header.child(
            div()
                .text_size(px(subtitle_style.size))
                .line_height(px(subtitle_style.line_height))
                .font_weight(subtitle_style.weight)
                .opacity(SUBTITLE_OPACITY)
                .child(subtitle),
        );
    }

    header
}

fn render_region(
    nodes: Vec<RenderedNavNode>,
    theme: &Arc<dyn NavigationSidebarTheme>,
    row_hovers: &mut impl Iterator<Item = NavigationSidebarHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = NavigationSidebarMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = NavigationSidebarClickHandler>,
) -> Div {
    let mut region = div().flex().flex_col().gap(px(REGION_GAP));

    for node in nodes {
        region = region.child(render_node(
            node,
            theme,
            row_hovers,
            row_mouse_downs,
            row_mouse_ups,
            row_mouse_up_outs,
            row_clicks,
        ));
    }

    region
}

#[allow(clippy::too_many_arguments)]
fn render_collapsed_rail_region(
    nodes: Vec<RenderedNavNode>,
    theme: &Arc<dyn NavigationSidebarTheme>,
    row_bounds: &mut impl Iterator<Item = NavigationSidebarBoundsHandler>,
    row_hovers: &mut impl Iterator<Item = NavigationSidebarHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = NavigationSidebarMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = NavigationSidebarClickHandler>,
) -> Div {
    let mut region = div().flex().flex_col().items_center().gap(px(REGION_GAP));

    for node in nodes {
        region = region.child(render_collapsed_rail_node(
            node,
            RowHandlers {
                bounds: row_bounds.next(),
                hover: row_hovers.next(),
                mouse_down: row_mouse_downs.next(),
                mouse_up: row_mouse_ups.next(),
                mouse_up_out: row_mouse_up_outs.next(),
                click: row_clicks.next(),
            },
            theme,
        ));
    }

    region
}

fn render_node(
    node: RenderedNavNode,
    theme: &Arc<dyn NavigationSidebarTheme>,
    row_hovers: &mut impl Iterator<Item = NavigationSidebarHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = NavigationSidebarMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = NavigationSidebarMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = NavigationSidebarClickHandler>,
) -> AnyElement {
    let RenderedNavNode { id, kind, label, icon, state, custom_element, focus_handle, has_children, children } = node;
    let expanded = state.expanded;
    let mut root = div().id(id.clone()).flex().flex_col().gap(px(REGION_GAP)).child(render_row(
        RowRenderInput { id, kind, label, icon, state, custom_element, focus_handle, has_children },
        RowHandlers {
            bounds: None,
            hover: row_hovers.next(),
            mouse_down: row_mouse_downs.next(),
            mouse_up: row_mouse_ups.next(),
            mouse_up_out: row_mouse_up_outs.next(),
            click: row_clicks.next(),
        },
        theme,
    ));

    if expanded {
        for child in children {
            root = root.child(render_node(
                child,
                theme,
                row_hovers,
                row_mouse_downs,
                row_mouse_ups,
                row_mouse_up_outs,
                row_clicks,
            ));
        }
    }

    root.into_any_element()
}

struct RowRenderInput {
    id: SharedString,
    kind: NavNodeKind,
    label: Option<SharedString>,
    icon: Option<LucideIcon>,
    state: super::NavNodeState,
    custom_element: Option<AnyElement>,
    focus_handle: Option<FocusHandle>,
    has_children: bool,
}

struct RowHandlers {
    bounds: Option<NavigationSidebarBoundsHandler>,
    hover: Option<NavigationSidebarHoverHandler>,
    mouse_down: Option<NavigationSidebarMouseDownHandler>,
    mouse_up: Option<NavigationSidebarMouseUpHandler>,
    mouse_up_out: Option<NavigationSidebarMouseUpHandler>,
    click: Option<NavigationSidebarClickHandler>,
}

fn render_row(input: RowRenderInput, handlers: RowHandlers, theme: &Arc<dyn NavigationSidebarTheme>) -> AnyElement {
    if let Some(element) = input.custom_element {
        return div().w_full().child(element).into_any_element();
    }

    match input.kind {
        NavNodeKind::Section => render_section_row(input.label.unwrap_or(input.id), theme).into_any_element(),
        NavNodeKind::Item => render_item_row(input, handlers, theme),
    }
}

fn render_section_row(label: SharedString, theme: &Arc<dyn NavigationSidebarTheme>) -> Div {
    let look = theme.resolve_section();

    div()
        .min_h(px(look.height))
        .pt(px(SECTION_PADDING_TOP))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .font_weight(look.typography.weight)
        .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1)])))
        .text_color(look.label_color)
        .child(label)
}

fn render_item_row(
    input: RowRenderInput,
    handlers: RowHandlers,
    theme: &Arc<dyn NavigationSidebarTheme>,
) -> AnyElement {
    let RowRenderInput { id, label, icon, state, focus_handle, has_children, .. } = input;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
    };
    let look = if has_children {
        theme.resolve_branch(interaction, ControlSize::Md)
    } else {
        theme.resolve_item(state.selected, interaction, ControlSize::Md)
    };
    let depth_indent = (look.icon_size + look.gap) * CHILD_DEPTH_INDENT_MULTIPLIER;
    let padding_left = look.padding_x + state.depth as f32 * depth_indent;
    let mut row = div()
        .id(format!("{id}-row"))
        .w_full()
        .min_h(px(look.height))
        .flex()
        .items_center()
        .gap(px(look.gap))
        .pl(px(padding_left))
        .pr(px(look.padding_x))
        .rounded(px(look.radius))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .text_color(look.foreground)
        .font_weight(look.typography.weight);

    if let Some(icon) = icon {
        row = row.child(render_lucide_icon(icon, look.icon_color, look.icon_size));
    }

    row = row.child(div().flex_1().child(label.unwrap_or(id)));

    if has_children {
        row = row.child(render_disclosure_icon(state.expanded, look.icon_color));
    }

    if let Some(background) = look.background {
        row = row.bg(background);
    }

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(DISABLED_ROW_OPACITY);
    }

    if let Some(focus_ring) = look.focus_ring {
        row = row.border_1().border_color(focus_ring);
    }

    let RowHandlers { hover, mouse_down, mouse_up, mouse_up_out, click, .. } = handlers;
    let mut row = row;
    if state.enabled {
        if let Some(focus_handle) = focus_handle.as_ref() {
            row = row.track_focus(focus_handle);
        }
        if let Some(hover) = hover {
            row = row.on_hover(hover);
        }
        if let Some(mouse_down) = mouse_down {
            row = row.on_mouse_down(MouseButton::Left, mouse_down);
        }
        if let Some(mouse_up) = mouse_up {
            row = row.on_mouse_up(MouseButton::Left, mouse_up);
        }
        if let Some(mouse_up_out) = mouse_up_out {
            row = row.on_mouse_up_out(MouseButton::Left, mouse_up_out);
        }
        if let Some(click) = click {
            row = row.on_click(click);
        }
    }

    row.into_any_element()
}

fn render_collapsed_rail_node(
    node: RenderedNavNode,
    handlers: RowHandlers,
    theme: &Arc<dyn NavigationSidebarTheme>,
) -> AnyElement {
    let RenderedNavNode { id, icon, state, focus_handle, has_children, .. } = node;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
    };
    let look = if has_children {
        theme.resolve_branch(interaction, ControlSize::Md)
    } else {
        theme.resolve_item(state.selected, interaction, ControlSize::Md)
    };
    let row_width = if has_children {
        rail_branch_button_width(look.height)
    } else {
        look.height
    };
    let mut row = div()
        .id(format!("{id}-rail-row"))
        .w(px(row_width))
        .h(px(look.height))
        .flex_none()
        .flex()
        .relative()
        .items_center()
        .justify_center()
        .rounded(px(look.radius))
        .text_color(look.foreground);

    if has_children {
        row = row.child(
            div()
                .absolute()
                .left(px(centered_icon_left(look.height, look.icon_size)))
                .top(px(centered_icon_left(look.height, look.icon_size)))
                .child(render_lucide_icon(
                    icon.expect("collapsed rail nodes always have icons"),
                    look.icon_color,
                    look.icon_size,
                )),
        );
        row = row.child(
            div()
                .absolute()
                .left(px(rail_branch_indicator_left(look.height)))
                .top(px(centered_icon_left(look.height, RAIL_BRANCH_INDICATOR_SIZE)))
                .child(render_lucide_icon(RAIL_BRANCH_INDICATOR_ICON, look.icon_color, RAIL_BRANCH_INDICATOR_SIZE)),
        );
    } else {
        row = row.child(render_lucide_icon(
            icon.expect("collapsed rail nodes always have icons"),
            look.icon_color,
            look.icon_size,
        ));
    }

    if let Some(background) = look.background {
        row = row.bg(background);
    }

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(DISABLED_ROW_OPACITY);
    }

    if let Some(focus_ring) = look.focus_ring {
        row = row.border_1().border_color(focus_ring);
    }

    let RowHandlers { bounds, hover, mouse_down, mouse_up, mouse_up_out, click } = handlers;
    if state.enabled {
        if let Some(focus_handle) = focus_handle.as_ref() {
            row = row.track_focus(focus_handle);
        }
        if let Some(hover) = hover {
            row = row.on_hover(hover);
        }
        if let Some(mouse_down) = mouse_down {
            row = row.on_mouse_down(MouseButton::Left, mouse_down);
        }
        if let Some(mouse_up) = mouse_up {
            row = row.on_mouse_up(MouseButton::Left, mouse_up);
        }
        if let Some(mouse_up_out) = mouse_up_out {
            row = row.on_mouse_up_out(MouseButton::Left, mouse_up_out);
        }
        if let Some(click) = click {
            row = row.on_click(click);
        }
    }

    if let Some(bounds) = bounds {
        div()
            .on_children_prepainted(move |child_bounds, window, cx| {
                if let Some(child_bounds) = child_bounds.first() {
                    bounds(child_bounds, window, cx);
                }
            })
            .child(row)
            .into_any_element()
    } else {
        row.into_any_element()
    }
}

fn render_collapse_trigger(
    trigger: RenderedCollapseTrigger,
    handlers: RowHandlers,
    theme: &Arc<dyn NavigationSidebarTheme>,
) -> AnyElement {
    let interaction = InteractionState {
        hovered: trigger.hovered,
        pressed: trigger.pressed,
        focused: trigger.focused,
        disabled: !trigger.enabled,
    };
    let look = theme.resolve_item(false, interaction, ControlSize::Md);
    let icon = if trigger.collapsed {
        COLLAPSE_COLLAPSED_ICON
    } else {
        COLLAPSE_EXPANDED_ICON
    };
    let mut row = div()
        .id(trigger.id)
        .size(px(look.height))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(look.radius))
        .text_color(look.foreground)
        .when(trigger.enabled, |row| row.cursor_pointer())
        .track_focus(&trigger.focus_handle)
        .child(render_lucide_icon(icon, look.icon_color, look.icon_size));

    if let Some(background) = look.background {
        row = row.bg(background);
    }

    if let Some(focus_ring) = look.focus_ring {
        row = row.border_1().border_color(focus_ring);
    }

    let RowHandlers { hover, mouse_down, mouse_up, mouse_up_out, click, .. } = handlers;
    if let Some(hover) = hover {
        row = row.on_hover(hover);
    }
    if let Some(mouse_down) = mouse_down {
        row = row.on_mouse_down(MouseButton::Left, mouse_down);
    }
    if let Some(mouse_up) = mouse_up {
        row = row.on_mouse_up(MouseButton::Left, mouse_up);
    }
    if let Some(mouse_up_out) = mouse_up_out {
        row = row.on_mouse_up_out(MouseButton::Left, mouse_up_out);
    }
    if let Some(click) = click {
        row = row.on_click(click);
    }

    row.into_any_element()
}

fn render_rail_submenu_overlay(
    submenu: RenderedRailSubmenu,
    look: FloatingMenuLook,
    item_hovers: Vec<FloatingMenuHoverHandler>,
    item_clicks: Vec<FloatingMenuClickHandler>,
) -> impl IntoElement {
    let menu = render_floating_menu(
        &submenu.id,
        &submenu.items,
        submenu.open_submenu,
        submenu.active_path,
        look.clone(),
        item_hovers,
        item_clicks,
    );

    anchored()
        .snap_to_window_with_margin(px(8.0))
        .anchor(Corner::TopLeft)
        .position(point(submenu.parent_bounds.right(), submenu.parent_bounds.top()))
        .offset(point(px(RAIL_SUBMENU_OFFSET_X), px(0.0)))
        .child(menu)
}

fn rail_branch_indicator_left(button_height: f32) -> f32 {
    button_height - RAIL_BRANCH_INDICATOR_RIGHT - RAIL_BRANCH_INDICATOR_SIZE
}

fn rail_branch_button_width(button_height: f32) -> f32 {
    button_height.max(rail_branch_indicator_left(button_height) + RAIL_BRANCH_INDICATOR_SIZE)
}

fn centered_icon_left(button_height: f32, icon_size: f32) -> f32 {
    (button_height - icon_size) * 0.5
}

fn render_disclosure_icon(expanded: bool, color: gpui::Hsla) -> AnyElement {
    let icon = if expanded {
        DISCLOSURE_EXPANDED_ICON
    } else {
        DISCLOSURE_COLLAPSED_ICON
    };

    render_lucide_icon(icon, color, DISCLOSURE_ICON_SIZE)
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family(LUCIDE_FONT_FAMILY)
        .font_weight(ICON_FONT_WEIGHT)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}
