use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontFeatures, FontWeight, IntoElement, Stateful,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::navigation_sidebar::{NavHostedContent, NavNode, NavNodeState, hosted_entity_presenter};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonRenderModel, ToggleButtonTemplate};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::{
    button, checkbox, context_menu, icon_button, introduction, palette, popup_menu, progress, radio_group, scrollbar,
    search, settings, shared::gallery_pane, slider, switch, tabs_navigation, toggle_button, toggle_group,
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
    Palette,
    Button,
    IconButton,
    ToggleButton,
    ToggleGroup,
    Switch,
    Checkbox,
    RadioGroup,
    Slider,
    Scrollbar,
    PopupMenu,
    ContextMenu,
    TabsNavigation,
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

#[derive(Clone)]
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
const PALETTE_PAGE: GalleryPage =
    GalleryPage { id: "palette", label: "Palette", icon: Some(LucideIcon::Palette), kind: GalleryPageKind::Palette };
const BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "button", label: "Button", icon: None, kind: GalleryPageKind::Button };
const ICON_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "icon-button", label: "Icon Button", icon: None, kind: GalleryPageKind::IconButton };
const TOGGLE_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "toggle-button", label: "Toggle Button", icon: None, kind: GalleryPageKind::ToggleButton };
const TOGGLE_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "toggle-group", label: "Toggle Group", icon: None, kind: GalleryPageKind::ToggleGroup };
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
const POPUP_MENU_PAGE: GalleryPage =
    GalleryPage { id: "popup-menu", label: "Popup Menu", icon: None, kind: GalleryPageKind::PopupMenu };
const CONTEXT_MENU_PAGE: GalleryPage =
    GalleryPage { id: "context-menu", label: "Context Menu", icon: None, kind: GalleryPageKind::ContextMenu };
const TABS_NAVIGATION_PAGE: GalleryPage =
    GalleryPage { id: "tabs-navigation", label: "Tabs Navigation", icon: None, kind: GalleryPageKind::TabsNavigation };
const PROGRESS_PAGE: GalleryPage =
    GalleryPage { id: "progress", label: "Progress", icon: None, kind: GalleryPageKind::Progress };
const SETTINGS_PAGE: GalleryPage = GalleryPage {
    id: "settings",
    label: "Settings",
    icon: Some(LucideIcon::Settings),
    kind: GalleryPageKind::Settings,
};

const PRIMARY_PAGES: &[GalleryPage] = &[INTRODUCTION_PAGE, SEARCH_PAGE, PALETTE_PAGE];
const BOTTOM_PAGES: &[GalleryPage] = &[SETTINGS_PAGE];
const COMMAND_PAGES: &[GalleryPage] = &[BUTTON_PAGE, ICON_BUTTON_PAGE, TOGGLE_BUTTON_PAGE];
const CHOICE_PAGES: &[GalleryPage] = &[SWITCH_PAGE, CHECKBOX_PAGE, RADIO_GROUP_PAGE, TOGGLE_GROUP_PAGE];
const INPUT_PAGES: &[GalleryPage] = &[SLIDER_PAGE, SCROLLBAR_PAGE];
const MENU_PAGES: &[GalleryPage] = &[POPUP_MENU_PAGE, CONTEXT_MENU_PAGE];
const NAVIGATION_PAGES: &[GalleryPage] = &[TABS_NAVIGATION_PAGE];
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
        id: "navigation",
        label: "Navigation",
        icon: LucideIcon::PanelTop,
        expanded: false,
        pages: NAVIGATION_PAGES,
    },
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
    pub(in crate::gallery) theme: GalleryThemePack,
    pub(super) button: button::ButtonPane,
    pub(super) icon_button: icon_button::IconButtonPane,
    pub(super) toggle_button: toggle_button::ToggleButtonPane,
    pub(super) toggle_group: toggle_group::ToggleGroupPane,
    pub(super) switch: switch::SwitchPane,
    pub(super) checkbox: checkbox::CheckboxPane,
    pub(super) radio_group: radio_group::RadioGroupPane,
    pub(super) slider: slider::SliderPane,
    pub(super) scrollbar: scrollbar::ScrollbarPane,
    pub(super) popup_menu: popup_menu::PopupMenuPane,
    pub(super) context_menu: context_menu::ContextMenuPane,
    pub(super) tabs_navigation: tabs_navigation::TabsNavigationPane,
    pub(super) progress: progress::ProgressPane,
}

impl GalleryPanes {
    pub(in crate::gallery) fn initial_selection() -> &'static str {
        INTRODUCTION_PAGE.id
    }

    pub(in crate::gallery) fn navigation(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> GalleryNavigation {
        let mut route_buttons = Vec::new();
        let mut branch_buttons = Vec::new();
        let mut nodes: Vec<NavNode> =
            PRIMARY_PAGES.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons, theme)).collect();

        let label_theme = theme.clone();
        nodes.push(NavNode::new("controls-label").content_presenter(
            move |state: &NavNodeState, window: &mut Window, cx: &mut App| {
                controls_label_presenter(state, window, cx, &label_theme)
            },
        ));

        nodes.extend(CONTROL_GROUPS.iter().map(|group| {
            let button = ToggleButton::new(format!("{}-branch", group.id))
                .label(group.label)
                .selected(group.expanded)
                .template(sidebar_disclosure_template(group.icon, theme))
                .spawn(cx);
            let focus_handle = focus_handle_for(&button, cx);
            branch_buttons.push(GalleryBranchButton { node_id: group.id, button: button.clone() });

            NavNode::new(group.id)
                .content_presenter(hosted_entity_presenter(button, focus_handle))
                .expanded(group.expanded)
                .children(group.pages.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons, theme)))
        }));

        let footer_nodes =
            BOTTOM_PAGES.iter().map(|page| nav_node_for_page(page, cx, &mut route_buttons, theme)).collect();

        GalleryNavigation { nodes, footer_nodes, route_buttons, branch_buttons }
    }

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            theme: theme.clone(),
            button: button::ButtonPane::new(cx, theme),
            icon_button: icon_button::IconButtonPane::new(cx, theme),
            toggle_button: toggle_button::ToggleButtonPane::new(cx, theme),
            toggle_group: toggle_group::ToggleGroupPane::new(cx, theme),
            switch: switch::SwitchPane::new(cx, theme),
            checkbox: checkbox::CheckboxPane::new(cx, theme),
            radio_group: radio_group::RadioGroupPane::new(cx, theme),
            slider: slider::SliderPane::new(cx, theme),
            scrollbar: scrollbar::ScrollbarPane::new(cx, theme),
            popup_menu: popup_menu::PopupMenuPane::new(cx, theme),
            context_menu: context_menu::ContextMenuPane::new(cx, theme),
            tabs_navigation: tabs_navigation::TabsNavigationPane::new(cx, theme),
            progress: progress::ProgressPane::new(cx, theme),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.button.subscribe(cx, subscriptions);
        self.icon_button.subscribe(cx, subscriptions);
        self.toggle_button.subscribe(cx, subscriptions);
        self.toggle_group.subscribe(cx, subscriptions);
        self.switch.subscribe(cx, subscriptions);
        self.checkbox.subscribe(cx, subscriptions);
        self.radio_group.subscribe(cx, subscriptions);
        self.slider.subscribe(cx, subscriptions);
        self.scrollbar.subscribe(cx, subscriptions);
        self.popup_menu.subscribe(cx, subscriptions);
        self.context_menu.subscribe(cx, subscriptions);
        self.tabs_navigation.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.button.notify_controls(cx);
        self.icon_button.notify_controls(cx);
        self.toggle_button.notify_controls(cx);
        self.toggle_group.notify_controls(cx);
        self.switch.notify_controls(cx);
        self.checkbox.notify_controls(cx);
        self.radio_group.notify_controls(cx);
        self.slider.notify_controls(cx);
        self.scrollbar.notify_controls(cx);
        self.popup_menu.notify_controls(cx);
        self.context_menu.notify_controls(cx);
        self.tabs_navigation.notify_controls(cx);
        self.progress.notify_controls(cx);
    }

    pub(in crate::gallery) fn render_selected(&self, selection: &str) -> AnyElement {
        let Some(page) = page_for_id(selection) else {
            debug_assert!(false, "unknown gallery page id: {selection}");
            return render_unknown_page(selection, &self.theme);
        };

        match page.kind {
            GalleryPageKind::Introduction => introduction::render(&self.theme),
            GalleryPageKind::Search => search::render(&self.theme),
            GalleryPageKind::Palette => palette::render(&self.theme),
            GalleryPageKind::Button => self.button.render(&self.theme),
            GalleryPageKind::IconButton => self.icon_button.render(&self.theme),
            GalleryPageKind::ToggleButton => self.toggle_button.render(&self.theme),
            GalleryPageKind::ToggleGroup => self.toggle_group.render(&self.theme),
            GalleryPageKind::Switch => self.switch.render(&self.theme),
            GalleryPageKind::Checkbox => self.checkbox.render(&self.theme),
            GalleryPageKind::RadioGroup => self.radio_group.render(&self.theme),
            GalleryPageKind::Slider => self.slider.render(&self.theme),
            GalleryPageKind::Scrollbar => self.scrollbar.render(&self.theme),
            GalleryPageKind::PopupMenu => self.popup_menu.render(&self.theme),
            GalleryPageKind::ContextMenu => self.context_menu.render(&self.theme),
            GalleryPageKind::TabsNavigation => self.tabs_navigation.render(&self.theme),
            GalleryPageKind::Progress => self.progress.render(&self.theme),
            GalleryPageKind::Settings => settings::render(&self.theme),
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
    theme: &GalleryThemePack,
) -> NavNode {
    let reserve_icon_space = PRIMARY_PAGES.iter().chain(BOTTOM_PAGES).all(|candidate| candidate.id != page.id);
    let button = ToggleButton::new(page.id)
        .label(page.label)
        .selected(page.id == INTRODUCTION_PAGE.id)
        .template(sidebar_leaf_template(page.icon, reserve_icon_space, theme))
        .spawn(cx);
    let focus_handle = focus_handle_for(&button, cx);

    route_buttons.push(GalleryRouteButton { page_id: page.id, button: button.clone() });

    NavNode::new(page.id).content_presenter(hosted_entity_presenter(button, focus_handle))
}

fn focus_handle_for<T: Focusable>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) -> FocusHandle {
    entity.read(cx).focus_handle(cx)
}

fn controls_label_presenter(
    _: &NavNodeState,
    _: &mut Window,
    _: &mut App,
    theme: &GalleryThemePack,
) -> NavHostedContent {
    let chrome = theme.chrome();

    NavHostedContent {
        element: div()
            .min_h(gpui::px(20.0))
            .pt(gpui::px(8.0))
            .text_size(gpui::px(11.0))
            .line_height(gpui::px(14.0))
            .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1)])))
            .text_color(chrome.section_label)
            .child("Controls")
            .into_any_element(),
        focus_handle: None,
    }
}

struct SidebarDisclosureTemplate {
    icon: LucideIcon,
    theme: GalleryThemePack,
}

impl ToggleButtonTemplate for SidebarDisclosureTemplate {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let chrome = self.theme.chrome();
        let foreground: gpui::Hsla = if model.enabled {
            chrome.body_text
        } else {
            chrome.section_label
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
            root = root.bg(self.theme.chrome().border);
        } else if model.state.hovered {
            root = root.bg(self.theme.chrome().content_background);
        }

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        if model.state.focused {
            root = root.border_1().border_color(chrome.focus_ring);
        }

        root
    }
}

fn sidebar_disclosure_template(icon: LucideIcon, theme: &GalleryThemePack) -> Arc<dyn ToggleButtonTemplate> {
    Arc::new(SidebarDisclosureTemplate { icon, theme: theme.clone() })
}

struct SidebarLeafTemplate {
    icon: Option<LucideIcon>,
    reserve_icon_space: bool,
    theme: GalleryThemePack,
}

impl ToggleButtonTemplate for SidebarLeafTemplate {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let chrome = self.theme.chrome();
        let metrics = &gpui_luma::theme::ThemeTokens::light().metrics.md;
        let background = if model.selected && model.state.pressed {
            Some(chrome.border)
        } else if model.selected && model.state.hovered {
            Some(chrome.content_background)
        } else if model.selected {
            Some(chrome.content_background)
        } else if model.state.pressed {
            Some(chrome.border)
        } else if model.state.hovered {
            Some(chrome.content_background)
        } else {
            None
        };
        let foreground = if !model.enabled {
            chrome.section_label
        } else if model.selected {
            chrome.title_text
        } else {
            chrome.body_text
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
            row = row.border_1().border_color(chrome.focus_ring);
        }

        row
    }
}

fn sidebar_leaf_template(
    icon: Option<LucideIcon>,
    reserve_icon_space: bool,
    theme: &GalleryThemePack,
) -> Arc<dyn ToggleButtonTemplate> {
    Arc::new(SidebarLeafTemplate { icon, reserve_icon_space, theme: theme.clone() })
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

fn render_unknown_page(selection: &str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    gallery_pane(
        "Unknown Page",
        div()
            .text_color(chrome.body_text)
            .child(format!("No gallery pane is registered for `{selection}`."))
            .into_any_element(),
        theme,
    )
}
