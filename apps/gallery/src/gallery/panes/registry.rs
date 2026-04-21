use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, IntoElement, Stateful, Subscription,
    Window, div, prelude::*, px, rgb,
};
use gpui_luma::controls::navigation_sidebar::{NavHostedContent, NavNode, NavNodeState, hosted_entity_presenter};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonRenderModel, ToggleButtonTemplate};
use gpui_luma::theme::ThemeTokens;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::{
    button, checkbox, context_menu, dropdown_menu, icon_button, introduction, progress, radio_group, scrollbar, search,
    settings, shared::gallery_pane, slider, switch, toggle_button,
};

#[derive(Clone, Copy)]
struct GalleryPage {
    id: &'static str,
    label: &'static str,
    icon: Option<LucideIcon>,
    kind: GalleryPageKind,
}

#[derive(Clone, Copy)]
enum GalleryPageKind {
    Introduction,
    Search,
    Button,
    IconButton,
    ToggleButton,
    Switch,
    Checkbox,
    RadioGroup,
    Slider,
    Scrollbar,
    DropdownMenu,
    ContextMenu,
    Progress,
    Settings,
}

struct GalleryNavGroup {
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    expanded: bool,
    pages: &'static [GalleryPage],
}

pub(in crate::gallery) struct GalleryNavigation {
    pub(in crate::gallery) nodes: Vec<NavNode>,
    pub(in crate::gallery) footer_nodes: Vec<NavNode>,
    pub(in crate::gallery) route_buttons: Vec<GalleryRouteButton>,
    pub(in crate::gallery) branch_buttons: Vec<GalleryBranchButton>,
}

#[derive(Clone)]
pub(in crate::gallery) struct GalleryRouteButton {
    pub(in crate::gallery) page_id: &'static str,
    pub(in crate::gallery) button: Entity<ToggleButton>,
}

pub(in crate::gallery) struct GalleryBranchButton {
    pub(in crate::gallery) node_id: &'static str,
    pub(in crate::gallery) button: Entity<ToggleButton>,
}

const INTRODUCTION_PAGE: GalleryPage = GalleryPage {
    id: "introduction",
    label: "Introduction",
    icon: Some(LucideIcon::BookOpenText),
    kind: GalleryPageKind::Introduction,
};
const SEARCH_PAGE: GalleryPage =
    GalleryPage { id: "search", label: "Search", icon: Some(LucideIcon::Search), kind: GalleryPageKind::Search };
const BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "button", label: "Button", icon: None, kind: GalleryPageKind::Button };
const ICON_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "icon-button", label: "Icon Button", icon: None, kind: GalleryPageKind::IconButton };
const TOGGLE_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "toggle-button", label: "Toggle Button", icon: None, kind: GalleryPageKind::ToggleButton };
const SWITCH_PAGE: GalleryPage =
    GalleryPage { id: "switch", label: "Switch", icon: None, kind: GalleryPageKind::Switch };
const CHECKBOX_PAGE: GalleryPage =
    GalleryPage { id: "checkbox", label: "Checkbox", icon: None, kind: GalleryPageKind::Checkbox };
const RADIO_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "radio-group", label: "Radio Group", icon: None, kind: GalleryPageKind::RadioGroup };
const SLIDER_PAGE: GalleryPage =
    GalleryPage { id: "slider", label: "Slider", icon: None, kind: GalleryPageKind::Slider };
const SCROLLBAR_PAGE: GalleryPage =
    GalleryPage { id: "scrollbar", label: "Scrollbar", icon: None, kind: GalleryPageKind::Scrollbar };
const DROPDOWN_MENU_PAGE: GalleryPage =
    GalleryPage { id: "dropdown-menu", label: "Dropdown Menu", icon: None, kind: GalleryPageKind::DropdownMenu };
const CONTEXT_MENU_PAGE: GalleryPage =
    GalleryPage { id: "context-menu", label: "Context Menu", icon: None, kind: GalleryPageKind::ContextMenu };
const PROGRESS_PAGE: GalleryPage =
    GalleryPage { id: "progress", label: "Progress", icon: None, kind: GalleryPageKind::Progress };
const SETTINGS_PAGE: GalleryPage = GalleryPage {
    id: "settings",
    label: "Settings",
    icon: Some(LucideIcon::Settings),
    kind: GalleryPageKind::Settings,
};

const PRIMARY_PAGES: &[GalleryPage] = &[INTRODUCTION_PAGE, SEARCH_PAGE];
const BOTTOM_PAGES: &[GalleryPage] = &[SETTINGS_PAGE];
const COMMAND_PAGES: &[GalleryPage] = &[BUTTON_PAGE, ICON_BUTTON_PAGE, TOGGLE_BUTTON_PAGE];
const CHOICE_PAGES: &[GalleryPage] = &[SWITCH_PAGE, CHECKBOX_PAGE, RADIO_GROUP_PAGE];
const INPUT_PAGES: &[GalleryPage] = &[SLIDER_PAGE, SCROLLBAR_PAGE];
const MENU_PAGES: &[GalleryPage] = &[DROPDOWN_MENU_PAGE, CONTEXT_MENU_PAGE];
const FEEDBACK_PAGES: &[GalleryPage] = &[PROGRESS_PAGE];
const CONTROL_GROUPS: &[GalleryNavGroup] = &[
    GalleryNavGroup {
        id: "command",
        label: "Command",
        icon: LucideIcon::Command,
        expanded: true,
        pages: COMMAND_PAGES,
    },
    GalleryNavGroup {
        id: "choice",
        label: "Choice",
        icon: LucideIcon::ListChecks,
        expanded: true,
        pages: CHOICE_PAGES,
    },
    GalleryNavGroup {
        id: "input",
        label: "Input",
        icon: LucideIcon::SlidersHorizontal,
        expanded: false,
        pages: INPUT_PAGES,
    },
    GalleryNavGroup { id: "menu", label: "Menu", icon: LucideIcon::Menu, expanded: false, pages: MENU_PAGES },
    GalleryNavGroup {
        id: "feedback",
        label: "Feedback",
        icon: LucideIcon::MessageSquare,
        expanded: false,
        pages: FEEDBACK_PAGES,
    },
];

#[derive(Clone)]
pub(in crate::gallery) struct GalleryPanes {
    pub(super) button: button::ButtonPane,
    pub(super) icon_button: icon_button::IconButtonPane,
    pub(super) toggle_button: toggle_button::ToggleButtonPane,
    pub(super) switch: switch::SwitchPane,
    pub(super) checkbox: checkbox::CheckboxPane,
    pub(super) radio_group: radio_group::RadioGroupPane,
    pub(super) slider: slider::SliderPane,
    pub(super) scrollbar: scrollbar::ScrollbarPane,
    pub(super) dropdown_menu: dropdown_menu::DropdownMenuPane,
    pub(super) context_menu: context_menu::ContextMenuPane,
    pub(super) progress: progress::ProgressPane,
}

impl GalleryPanes {
    pub(in crate::gallery) fn initial_selection() -> &'static str {
        INTRODUCTION_PAGE.id
    }

    pub(in crate::gallery) fn navigation(cx: &mut Context<GalleryApp>) -> GalleryNavigation {
        let mut route_buttons = Vec::new();
        let mut branch_buttons = Vec::new();
        let mut nodes: Vec<NavNode> =
            PRIMARY_PAGES.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons)).collect();

        nodes.push(NavNode::new("controls-label").content_presenter(controls_label_presenter));

        nodes.extend(CONTROL_GROUPS.iter().map(|group| {
            let button = ToggleButton::new(format!("{}-branch", group.id))
                .label(group.label)
                .selected(group.expanded)
                .template(sidebar_disclosure_template(group.icon))
                .spawn(cx);
            let focus_handle = focus_handle_for(&button, cx);
            branch_buttons.push(GalleryBranchButton { node_id: group.id, button: button.clone() });

            NavNode::new(group.id)
                .content_presenter(hosted_entity_presenter(button, focus_handle))
                .expanded(group.expanded)
                .children(group.pages.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons)))
        }));

        let footer_nodes = BOTTOM_PAGES.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons)).collect();

        GalleryNavigation { nodes, footer_nodes, route_buttons, branch_buttons }
    }

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            button: button::ButtonPane::new(cx),
            icon_button: icon_button::IconButtonPane::new(cx),
            toggle_button: toggle_button::ToggleButtonPane::new(cx),
            switch: switch::SwitchPane::new(cx),
            checkbox: checkbox::CheckboxPane::new(cx),
            radio_group: radio_group::RadioGroupPane::new(cx),
            slider: slider::SliderPane::new(cx),
            scrollbar: scrollbar::ScrollbarPane::new(cx),
            dropdown_menu: dropdown_menu::DropdownMenuPane::new(cx),
            context_menu: context_menu::ContextMenuPane::new(cx),
            progress: progress::ProgressPane::new(cx),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.button.subscribe(cx, subscriptions);
        self.icon_button.subscribe(cx, subscriptions);
        self.toggle_button.subscribe(cx, subscriptions);
        self.switch.subscribe(cx, subscriptions);
        self.checkbox.subscribe(cx, subscriptions);
        self.radio_group.subscribe(cx, subscriptions);
        self.slider.subscribe(cx, subscriptions);
        self.scrollbar.subscribe(cx, subscriptions);
        self.dropdown_menu.subscribe(cx, subscriptions);
        self.context_menu.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn render_selected(&self, selection: &str) -> AnyElement {
        let Some(page) = page_for_id(selection) else {
            debug_assert!(false, "unknown gallery page id: {selection}");
            return render_unknown_page(selection);
        };

        match page.kind {
            GalleryPageKind::Introduction => introduction::render(),
            GalleryPageKind::Search => search::render(),
            GalleryPageKind::Button => self.button.render(),
            GalleryPageKind::IconButton => self.icon_button.render(),
            GalleryPageKind::ToggleButton => self.toggle_button.render(),
            GalleryPageKind::Switch => self.switch.render(),
            GalleryPageKind::Checkbox => self.checkbox.render(),
            GalleryPageKind::RadioGroup => self.radio_group.render(),
            GalleryPageKind::Slider => self.slider.render(),
            GalleryPageKind::Scrollbar => self.scrollbar.render(),
            GalleryPageKind::DropdownMenu => self.dropdown_menu.render(),
            GalleryPageKind::ContextMenu => self.context_menu.render(),
            GalleryPageKind::Progress => self.progress.render(),
            GalleryPageKind::Settings => settings::render(),
        }
    }
}

fn page_for_id(id: &str) -> Option<GalleryPage> {
    PRIMARY_PAGES
        .iter()
        .chain(BOTTOM_PAGES)
        .chain(CONTROL_GROUPS.iter().flat_map(|group| group.pages.iter()))
        .copied()
        .find(|page| page.id == id)
}

fn nav_node_for_page(
    page: &GalleryPage,
    cx: &mut Context<GalleryApp>,
    route_buttons: &mut Vec<GalleryRouteButton>,
) -> NavNode {
    let reserve_icon_space = PRIMARY_PAGES.iter().chain(BOTTOM_PAGES).all(|candidate| candidate.id != page.id);
    let button = ToggleButton::new(page.id)
        .label(page.label)
        .selected(page.id == INTRODUCTION_PAGE.id)
        .template(sidebar_leaf_template(page.icon, reserve_icon_space))
        .spawn(cx);
    let focus_handle = focus_handle_for(&button, cx);

    route_buttons.push(GalleryRouteButton { page_id: page.id, button: button.clone() });

    NavNode::new(page.id).content_presenter(hosted_entity_presenter(button, focus_handle))
}

fn focus_handle_for<T: Focusable>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) -> FocusHandle {
    entity.read(cx).focus_handle(cx)
}

fn controls_label_presenter(_: &NavNodeState, _: &mut Window, _: &mut App) -> NavHostedContent {
    NavHostedContent {
        element: div()
            .min_h(gpui::px(20.0))
            .pt(gpui::px(8.0))
            .text_size(gpui::px(11.0))
            .line_height(gpui::px(14.0))
            .text_color(rgb(0x94a3b8))
            .child("Controls")
            .into_any_element(),
        focus_handle: None,
    }
}

struct SidebarDisclosureTemplate {
    icon: LucideIcon,
}

impl ToggleButtonTemplate for SidebarDisclosureTemplate {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let foreground: gpui::Hsla = if model.enabled {
            rgb(0x334155).into()
        } else {
            rgb(0x94a3b8).into()
        };
        let mut root = div()
            .id(model.id.clone())
            .w_full()
            .min_h(px(30.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .px(px(8.0))
            .rounded(px(6.0))
            .text_size(px(13.0))
            .line_height(px(18.0))
            .text_color(foreground)
            .font_weight(FontWeight::MEDIUM)
            .child(render_lucide_icon(self.icon, foreground, 16.0))
            .child(div().flex_1().child(model.label.clone()));

        if model.state.pressed {
            root = root.bg(rgb(0xe2e8f0));
        } else if model.state.hovered {
            root = root.bg(rgb(0xf1f5f9));
        }

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        if model.state.focused {
            root = root.border_1().border_color(rgb(0xf59e0b));
        }

        root
    }
}

fn sidebar_disclosure_template(icon: LucideIcon) -> Arc<dyn ToggleButtonTemplate> {
    Arc::new(SidebarDisclosureTemplate { icon })
}

struct SidebarLeafTemplate {
    icon: Option<LucideIcon>,
    reserve_icon_space: bool,
    tokens: ThemeTokens,
}

impl ToggleButtonTemplate for SidebarLeafTemplate {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics.md;
        let background = if model.selected && model.state.pressed {
            Some(rgb(0xc5cede).into())
        } else if model.selected && model.state.hovered {
            Some(rgb(0xcbd4e4).into())
        } else if model.selected {
            Some(rgb(0xd6deee).into())
        } else if model.state.pressed {
            Some(colors.surface_pressed)
        } else if model.state.hovered {
            Some(colors.surface_hover)
        } else {
            None
        };
        let foreground = if !model.enabled {
            colors.text_disabled
        } else if model.selected {
            rgb(0x1f2937).into()
        } else {
            colors.text
        };
        let padding_left = if self.reserve_icon_space { 26.0 } else { 8.0 };
        let placeholder_size = if self.reserve_icon_space { 16.0 } else { 0.0 };
        let mut row = div()
            .id(model.id.clone())
            .w_full()
            .flex()
            .items_center()
            .gap(px(metrics.gap))
            .min_h(px(30.0))
            .pl(px(padding_left))
            .pr(px(8.0))
            .rounded(px(metrics.radius))
            .text_size(px(13.0))
            .line_height(px(18.0))
            .text_color(foreground)
            .child(match self.icon {
                Some(icon) => render_lucide_icon(icon, foreground, 16.0),
                None => div().size(px(placeholder_size)).into_any_element(),
            })
            .child(div().flex_1().child(model.label.clone()));

        if let Some(background) = background {
            row = row.bg(background);
        }

        if model.enabled {
            row = row.cursor_pointer();
        } else {
            row = row.opacity(0.56);
        }

        if model.state.focused {
            row = row.border_1().border_color(rgb(0xf59e0b));
        }

        row
    }
}

fn sidebar_leaf_template(icon: Option<LucideIcon>, reserve_icon_space: bool) -> Arc<dyn ToggleButtonTemplate> {
    Arc::new(SidebarLeafTemplate { icon, reserve_icon_space, tokens: ThemeTokens::default() })
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

fn render_unknown_page(selection: &str) -> AnyElement {
    gallery_pane(
        "Unknown Page",
        div()
            .text_color(rgb(0x334155))
            .child(format!("No gallery pane is registered for `{selection}`."))
            .into_any_element(),
    )
}
