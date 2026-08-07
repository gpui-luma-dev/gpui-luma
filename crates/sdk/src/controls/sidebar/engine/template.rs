use std::sync::{Arc, OnceLock};

use gpui::{
    Anchor, AnyElement, App, Bounds, ClickEvent, Div, FocusHandle, FontFeatures, FontWeight, MouseButton,
    MouseDownEvent, MouseUpEvent, Pixels, SharedString, Stateful, Window, anchored, deferred, div, point, prelude::*,
    px,
};
use lucide_icons::Icon as LucideIcon;

use super::{NavNodeKind, SidebarPanelEngineRenderModel, RenderedCollapseTrigger, RenderedNavNode, RenderedRailSubmenu};
use crate::controls::floating_menu::{FloatingMenuClickHandler, FloatingMenuHoverHandler, render_floating_menu};
use crate::controls::scroll_container::ScrollContainer;
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, LumaTypography};
use crate::controls::floating_menu::{FloatingMenuLook, FloatingMenuTheme, default_floating_menu_theme};
use super::{SidebarTheme, default_sidebar_theme};

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

pub type SidebarPanelBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + 'static>;
pub type SidebarPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SidebarPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type SidebarPanelMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type SidebarPanelMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(Default)]
pub struct SidebarPanelTemplateHandlers {
    pub collapse_hover: Option<SidebarPanelHoverHandler>,
    pub collapse_mouse_down: Option<SidebarPanelMouseDownHandler>,
    pub collapse_mouse_up: Option<SidebarPanelMouseUpHandler>,
    pub collapse_mouse_up_out: Option<SidebarPanelMouseUpHandler>,
    pub collapse_click: Option<SidebarPanelClickHandler>,
    pub row_bounds: Vec<SidebarPanelBoundsHandler>,
    pub row_hovers: Vec<SidebarPanelHoverHandler>,
    pub row_mouse_downs: Vec<SidebarPanelMouseDownHandler>,
    pub row_mouse_ups: Vec<SidebarPanelMouseUpHandler>,
    pub row_mouse_up_outs: Vec<SidebarPanelMouseUpHandler>,
    pub row_clicks: Vec<SidebarPanelClickHandler>,
    pub rail_submenu_mouse_down_out: Option<SidebarPanelMouseDownHandler>,
    pub rail_submenu_item_hovers: Vec<FloatingMenuHoverHandler>,
    pub rail_submenu_item_clicks: Vec<FloatingMenuClickHandler>,
}

pub type SidebarPanelTemplateModifier = Box<dyn Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait SidebarPanelTemplate: Send + Sync {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSidebarPanelTemplate {
    theme: Arc<dyn SidebarTheme>,
    floating_menu_theme: Arc<dyn FloatingMenuTheme>,
    modifiers: Vec<SidebarPanelTemplateModifier>,
}

impl ThemedSidebarPanelTemplate {
    pub fn new(theme: Arc<dyn SidebarTheme>) -> Self {
        Self { theme, floating_menu_theme: default_floating_menu_theme(), modifiers: Vec::new() }
    }

    pub fn new_with_floating_menu_theme(
        theme: Arc<dyn SidebarTheme>,
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

struct ModifiedSidebarPanelTemplate {
    base: Arc<dyn SidebarPanelTemplate>,
    modifiers: Vec<SidebarPanelTemplateModifier>,
}

impl ModifiedSidebarPanelTemplate {
    fn new(base: Arc<dyn SidebarPanelTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: SidebarPanelTemplateModifier) -> Self {
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

impl Default for ThemedSidebarPanelTemplate {
    fn default() -> Self {
        Self::new(default_sidebar_theme())
    }
}

pub fn default_sidebar_panel_template() -> Arc<dyn SidebarPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SidebarPanelTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSidebarPanelTemplate::default())).clone()
}

pub(crate) fn modified_sidebar_panel_template<F>(
    template: Arc<dyn SidebarPanelTemplate>,
    modifier: F,
) -> Arc<dyn SidebarPanelTemplate>
where
    F: Fn(Stateful<Div>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedSidebarPanelTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl SidebarPanelTemplate for ModifiedSidebarPanelTemplate {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, main_scroll, handlers, window, cx);
        self.apply_modifiers(root)
    }
}

impl SidebarPanelTemplate for ThemedSidebarPanelTemplate {
    fn render(
        &self,
        model: SidebarPanelEngineRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let container = self.theme.resolve_container();
        let SidebarPanelTemplateHandlers {
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
    theme: &Arc<dyn SidebarTheme>,
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
    theme: &Arc<dyn SidebarTheme>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
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
    theme: &Arc<dyn SidebarTheme>,
    row_bounds: &mut impl Iterator<Item = SidebarPanelBoundsHandler>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
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
    theme: &Arc<dyn SidebarTheme>,
    row_hovers: &mut impl Iterator<Item = SidebarPanelHoverHandler>,
    row_mouse_downs: &mut impl Iterator<Item = SidebarPanelMouseDownHandler>,
    row_mouse_ups: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_mouse_up_outs: &mut impl Iterator<Item = SidebarPanelMouseUpHandler>,
    row_clicks: &mut impl Iterator<Item = SidebarPanelClickHandler>,
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
    bounds: Option<SidebarPanelBoundsHandler>,
    hover: Option<SidebarPanelHoverHandler>,
    mouse_down: Option<SidebarPanelMouseDownHandler>,
    mouse_up: Option<SidebarPanelMouseUpHandler>,
    mouse_up_out: Option<SidebarPanelMouseUpHandler>,
    click: Option<SidebarPanelClickHandler>,
}

fn render_row(input: RowRenderInput, handlers: RowHandlers, theme: &Arc<dyn SidebarTheme>) -> AnyElement {
    if let Some(element) = input.custom_element {
        return div().w_full().child(element).into_any_element();
    }

    match input.kind {
        NavNodeKind::Section => render_section_row(input.label.unwrap_or(input.id), theme).into_any_element(),
        NavNodeKind::Item => render_item_row(input, handlers, theme),
    }
}

fn render_section_row(label: SharedString, theme: &Arc<dyn SidebarTheme>) -> Div {
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

fn render_item_row(input: RowRenderInput, handlers: RowHandlers, theme: &Arc<dyn SidebarTheme>) -> AnyElement {
    let RowRenderInput { id, label, icon, state, focus_handle, has_children, .. } = input;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
        invalid: false,
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

    if let Some(focus_border) = look.focus_border {
        row = row.border_1().border_color(focus_border);
    }

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(DISABLED_ROW_OPACITY);
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
    theme: &Arc<dyn SidebarTheme>,
) -> AnyElement {
    let RenderedNavNode { id, icon, state, focus_handle, has_children, .. } = node;
    let interaction = InteractionState {
        hovered: state.hovered,
        pressed: state.pressed,
        focused: state.focused,
        disabled: !state.enabled,
        invalid: false,
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

    if let Some(focus_border) = look.focus_border {
        row = row.border_1().border_color(focus_border);
    }

    if state.enabled {
        row = row.cursor_pointer();
    } else {
        row = row.opacity(DISABLED_ROW_OPACITY);
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
    theme: &Arc<dyn SidebarTheme>,
) -> AnyElement {
    let interaction = InteractionState {
        hovered: trigger.hovered,
        pressed: trigger.pressed,
        focused: trigger.focused,
        disabled: !trigger.enabled,
        invalid: false,
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
        .anchor(Anchor::TopLeft)
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
