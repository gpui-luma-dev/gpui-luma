use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontFeatures, FontWeight, IntoElement, Stateful,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::navigation_sidebar::{NavHostedContent, NavNode, NavNodeState, entity_presenter};
use gpui_luma::controls::command::button::{Button, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::navigation_sidebar::NavigationSidebarTheme;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::{
    autocomplete, button, checkbox, choice_controls_template, combobox, context_menu, floating_menu, icon_button,
    introduction, listbox, navigation_sidebar, palette, popup_menu, progress, prototypes, radio_button, radio_group,
    scrollbar, search, search_selector, selection_panel, selector, selector_controls_template, settings,
    shared::gallery_pane, slider, switch, tabs_navigation, textarea, textfield, theme_usage, toggle, toggle_group,
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
    ThemeUsage,
    Button,
    DecoratedButton,
    AutocompleteTextField,
    ComboBox,
    SearchSelector,
    Selector,
    SelectionPanel,
    SelectorTemplates,
    CustomButton,
    IconButton,
    Toggle,
    ToggleGroup,
    Switch,
    Checkbox,
    RadioButton,
    RadioGroup,
    ChoiceTemplates,
    ListBox,
    Slider,
    Scrollbar,
    TextArea,
    TextField,
    FloatingMenu,
    PopupMenu,
    ContextMenu,
    NavigationSidebar,
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
    pub(in crate::gallery) button: Entity<Button<bool>>,
}

#[derive(Clone)]
pub(in crate::gallery) struct GalleryBranchButton {
    pub(in crate::gallery) node_id: &'static str,
    pub(in crate::gallery) button: Entity<Button<bool>>,
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
const THEME_USAGE_PAGE: GalleryPage = GalleryPage {
    id: "theme-usage",
    label: "Theme Usage",
    icon: Some(LucideIcon::ListTree),
    kind: GalleryPageKind::ThemeUsage,
};
const BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "button", label: "Button", icon: None, kind: GalleryPageKind::Button };
const ICON_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "icon-button", label: "Icon Button", icon: None, kind: GalleryPageKind::IconButton };
const TOGGLE_PAGE: GalleryPage =
    GalleryPage { id: "toggle", label: "Toggle", icon: None, kind: GalleryPageKind::Toggle };
const TOGGLE_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "toggle-group", label: "Toggle Group", icon: None, kind: GalleryPageKind::ToggleGroup };
const SWITCH_PAGE: GalleryPage =
    GalleryPage { id: "switch", label: "Switch", icon: None, kind: GalleryPageKind::Switch };
const CHECKBOX_PAGE: GalleryPage =
    GalleryPage { id: "checkbox", label: "Checkbox", icon: None, kind: GalleryPageKind::Checkbox };
const RADIO_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "radio-button", label: "Radio Button", icon: None, kind: GalleryPageKind::RadioButton };
const RADIO_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "radio-group", label: "Radio Group", icon: None, kind: GalleryPageKind::RadioGroup };
const CHOICE_TEMPLATES_PAGE: GalleryPage = GalleryPage {
    id: "choice-templates",
    label: "Choice Templates",
    icon: None,
    kind: GalleryPageKind::ChoiceTemplates,
};
const LISTBOX_PAGE: GalleryPage =
    GalleryPage { id: "listbox", label: "ListBox", icon: None, kind: GalleryPageKind::ListBox };
const SLIDER_PAGE: GalleryPage =
    GalleryPage { id: "slider", label: "Slider", icon: None, kind: GalleryPageKind::Slider };
const SCROLLBAR_PAGE: GalleryPage =
    GalleryPage { id: "scrollbar", label: "Scrollbar", icon: None, kind: GalleryPageKind::Scrollbar };
const TEXTAREA_PAGE: GalleryPage =
    GalleryPage { id: "textarea", label: "Text Area", icon: None, kind: GalleryPageKind::TextArea };
const TEXTFIELD_PAGE: GalleryPage =
    GalleryPage { id: "textfield", label: "Text Field", icon: None, kind: GalleryPageKind::TextField };
const FLOATING_MENU_PAGE: GalleryPage =
    GalleryPage { id: "floating-menu", label: "Floating Menu", icon: None, kind: GalleryPageKind::FloatingMenu };
const POPUP_MENU_PAGE: GalleryPage =
    GalleryPage { id: "popup-menu", label: "Popup Menu", icon: None, kind: GalleryPageKind::PopupMenu };
const CONTEXT_MENU_PAGE: GalleryPage =
    GalleryPage { id: "context-menu", label: "Context Menu", icon: None, kind: GalleryPageKind::ContextMenu };
const NAVIGATION_SIDEBAR_PAGE: GalleryPage = GalleryPage {
    id: "navigation-sidebar",
    label: "Navigation Sidebar",
    icon: None,
    kind: GalleryPageKind::NavigationSidebar,
};
const TABS_NAVIGATION_PAGE: GalleryPage =
    GalleryPage { id: "tabs-navigation", label: "Tabs Navigation", icon: None, kind: GalleryPageKind::TabsNavigation };
const PROGRESS_PAGE: GalleryPage =
    GalleryPage { id: "progress", label: "Progress", icon: None, kind: GalleryPageKind::Progress };
const DECORATED_BUTTON_PAGE: GalleryPage = GalleryPage {
    id: "decorated-button",
    label: "Decorated Button",
    icon: None,
    kind: GalleryPageKind::DecoratedButton,
};
const CUSTOM_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "custom-button", label: "Custom Button", icon: None, kind: GalleryPageKind::CustomButton };
const AUTOCOMPLETE_TEXTFIELD_PAGE: GalleryPage = GalleryPage {
    id: "autocomplete-textfield",
    label: "Autocomplete TextBox",
    icon: None,
    kind: GalleryPageKind::AutocompleteTextField,
};
const COMBOBOX_PAGE: GalleryPage =
    GalleryPage { id: "combobox", label: "ComboBox", icon: None, kind: GalleryPageKind::ComboBox };
const SEARCH_SELECTOR_PAGE: GalleryPage =
    GalleryPage { id: "search-selector", label: "SearchSelector", icon: None, kind: GalleryPageKind::SearchSelector };
const SELECTOR_PAGE: GalleryPage =
    GalleryPage { id: "popup-selector", label: "Selector", icon: None, kind: GalleryPageKind::Selector };
const SELECTION_PANEL_PAGE: GalleryPage =
    GalleryPage { id: "selection-panel", label: "Selection Panel", icon: None, kind: GalleryPageKind::SelectionPanel };
const SELECTOR_TEMPLATES_PAGE: GalleryPage = GalleryPage {
    id: "selector-templates",
    label: "Selector Templates",
    icon: None,
    kind: GalleryPageKind::SelectorTemplates,
};
const SETTINGS_PAGE: GalleryPage = GalleryPage {
    id: "settings",
    label: "Settings",
    icon: Some(LucideIcon::Settings),
    kind: GalleryPageKind::Settings,
};

const PRIMARY_PAGES: &[GalleryPage] = &[INTRODUCTION_PAGE, SEARCH_PAGE, PALETTE_PAGE, THEME_USAGE_PAGE];
const BOTTOM_PAGES: &[GalleryPage] = &[SETTINGS_PAGE];
const COMMAND_PAGES: &[GalleryPage] = &[BUTTON_PAGE, ICON_BUTTON_PAGE, CUSTOM_BUTTON_PAGE];
const CHOICE_PAGES: &[GalleryPage] = &[
    TOGGLE_PAGE,
    SWITCH_PAGE,
    CHECKBOX_PAGE,
    RADIO_BUTTON_PAGE,
    RADIO_GROUP_PAGE,
    CHOICE_TEMPLATES_PAGE,
    LISTBOX_PAGE,
    TOGGLE_GROUP_PAGE,
];
const INPUT_PAGES: &[GalleryPage] = &[TEXTFIELD_PAGE, TEXTAREA_PAGE, SLIDER_PAGE, SCROLLBAR_PAGE];
const MENU_PAGES: &[GalleryPage] = &[FLOATING_MENU_PAGE, POPUP_MENU_PAGE, CONTEXT_MENU_PAGE];
const NAVIGATION_PAGES: &[GalleryPage] = &[NAVIGATION_SIDEBAR_PAGE, TABS_NAVIGATION_PAGE];
const FEEDBACK_PAGES: &[GalleryPage] = &[PROGRESS_PAGE];
const SELECTION_PAGES: &[GalleryPage] = &[
    AUTOCOMPLETE_TEXTFIELD_PAGE,
    COMBOBOX_PAGE,
    SEARCH_SELECTOR_PAGE,
    SELECTOR_PAGE,
    SELECTION_PANEL_PAGE,
    SELECTOR_TEMPLATES_PAGE,
];
const PROTOTYPES_PAGES: &[GalleryPage] = &[DECORATED_BUTTON_PAGE];
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
    GalleryNavGroup {
        id: "selection",
        label: "Selection",
        icon: LucideIcon::ListFilter,
        expanded: false,
        pages: SELECTION_PAGES,
    },
    GalleryNavGroup {
        id: "prototypes",
        label: "Prototypes",
        icon: LucideIcon::Command,
        expanded: false,
        pages: PROTOTYPES_PAGES,
    },
];

#[derive(Clone)]
pub(in crate::gallery) struct GalleryPanes {
    pub(in crate::gallery) theme: GalleryThemePack,
    pub(super) introduction: introduction::IntroductionPane,
    pub(super) decorated_button: prototypes::ButtonPane,
    pub(super) autocomplete_textfield: autocomplete::AutocompleteTextFieldPane,
    pub(super) combobox: combobox::ComboBoxPane,
    pub(super) search_selector: search_selector::SearchSelectorPane,
    pub(super) selector: selector::SelectorPane,
    pub(super) selection_panel: selection_panel::SelectionPanelPane,
    pub(super) selector_templates: selector_controls_template::SelectorControlsTemplatePane,
    pub(super) custom_button: prototypes::ModButtonPane,
    pub(super) button: button::ButtonPane,
    pub(super) icon_button: icon_button::IconButtonPane,
    pub(super) toggle: toggle::TogglePane,
    pub(super) toggle_group: toggle_group::ToggleGroupPane,
    pub(super) switch: switch::SwitchPane,
    pub(super) checkbox: checkbox::CheckboxPane,
    pub(super) radio_button: radio_button::RadioButtonPane,
    pub(super) radio_group: radio_group::RadioGroupPane,
    pub(super) choice_templates: choice_controls_template::ChoiceControlsTemplatePane,
    pub(super) listbox: listbox::ListBoxPane,
    pub(super) slider: slider::SliderPane,
    pub(super) scrollbar: scrollbar::ScrollbarPane,
    pub(super) textarea: textarea::TextAreaPane,
    pub(super) textfield: textfield::TextFieldPane,
    pub(super) floating_menu: floating_menu::FloatingMenuPane,
    pub(super) popup_menu: popup_menu::PopupMenuPane,
    pub(super) context_menu: context_menu::ContextMenuPane,
    pub(super) navigation_sidebar: navigation_sidebar::NavigationSidebarPane,
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

        let label_theme = theme.navigation_sidebar_theme();
        nodes.push(NavNode::new("controls-label").presenter(
            move |state: &NavNodeState, window: &mut Window, cx: &mut App| {
                controls_label_presenter(state, window, cx, label_theme.clone())
            },
        ));

        nodes.extend(CONTROL_GROUPS.iter().map(|group| {
            let label = group.label;
            let button = Button::new(format!("{}-branch", group.id))
                .typed(group.expanded)
                .content(move |_, _| div().child(label).into_any_element())
                .template(sidebar_disclosure_template(group.icon, theme))
                .spawn(cx);
            let focus_handle = focus_handle_for(&button, cx);
            branch_buttons.push(GalleryBranchButton { node_id: group.id, button: button.clone() });

            NavNode::new(group.id)
                .label(group.label)
                .icon(group.icon)
                .presenter(entity_presenter(button, focus_handle))
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
            introduction: introduction::IntroductionPane::new(cx, theme),
            decorated_button: prototypes::ButtonPane::new(cx, theme),
            autocomplete_textfield: autocomplete::AutocompleteTextFieldPane::new(cx, theme),
            combobox: combobox::ComboBoxPane::new(cx, theme),
            search_selector: search_selector::SearchSelectorPane::new(cx, theme),
            selector: selector::SelectorPane::new(cx, theme),
            selection_panel: selection_panel::SelectionPanelPane::new(cx, theme),
            selector_templates: selector_controls_template::SelectorControlsTemplatePane::new(cx, theme),
            custom_button: prototypes::ModButtonPane::new(cx, theme),
            button: button::ButtonPane::new(cx, theme),
            icon_button: icon_button::IconButtonPane::new(cx, theme),
            toggle: toggle::TogglePane::new(cx, theme),
            toggle_group: toggle_group::ToggleGroupPane::new(cx, theme),
            switch: switch::SwitchPane::new(cx, theme),
            checkbox: checkbox::CheckboxPane::new(cx, theme),
            radio_button: radio_button::RadioButtonPane::new(cx, theme),
            radio_group: radio_group::RadioGroupPane::new(cx, theme),
            choice_templates: choice_controls_template::ChoiceControlsTemplatePane::new(cx, theme),
            listbox: listbox::ListBoxPane::new(cx, theme),
            slider: slider::SliderPane::new(cx, theme),
            scrollbar: scrollbar::ScrollbarPane::new(cx, theme),
            textarea: textarea::TextAreaPane::new(cx, theme),
            textfield: textfield::TextFieldPane::new(cx, theme),
            floating_menu: floating_menu::FloatingMenuPane::new(cx),
            popup_menu: popup_menu::PopupMenuPane::new(cx, theme),
            context_menu: context_menu::ContextMenuPane::new(cx, theme),
            navigation_sidebar: navigation_sidebar::NavigationSidebarPane::new(cx, theme),
            tabs_navigation: tabs_navigation::TabsNavigationPane::new(cx, theme),
            progress: progress::ProgressPane::new(cx, theme),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.introduction.subscribe(cx, subscriptions);
        self.decorated_button.subscribe(cx, subscriptions);
        self.autocomplete_textfield.subscribe(cx, subscriptions);
        self.combobox.subscribe(cx, subscriptions);
        self.search_selector.subscribe(cx, subscriptions);
        self.selector.subscribe(cx, subscriptions);
        self.selection_panel.subscribe(cx, subscriptions);
        self.custom_button.subscribe(cx, subscriptions);
        self.button.subscribe(cx, subscriptions);
        self.icon_button.subscribe(cx, subscriptions);
        self.toggle.subscribe(cx, subscriptions);
        self.toggle_group.subscribe(cx, subscriptions);
        self.switch.subscribe(cx, subscriptions);
        self.checkbox.subscribe(cx, subscriptions);
        self.radio_button.subscribe(cx, subscriptions);
        self.radio_group.subscribe(cx, subscriptions);
        self.listbox.subscribe(cx, subscriptions);
        self.slider.subscribe(cx, subscriptions);
        self.scrollbar.subscribe(cx, subscriptions);
        self.textarea.subscribe(cx, subscriptions);
        self.textfield.subscribe(cx, subscriptions);
        self.floating_menu.subscribe(cx, subscriptions);
        self.popup_menu.subscribe(cx, subscriptions);
        self.context_menu.subscribe(cx, subscriptions);
        self.navigation_sidebar.subscribe(cx, subscriptions);
        self.tabs_navigation.subscribe(cx, subscriptions);
    }

    #[allow(dead_code)]
    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.introduction.notify_controls(cx);
        self.decorated_button.notify_controls(cx);
        self.autocomplete_textfield.notify_controls(cx);
        self.combobox.notify_controls(cx);
        self.search_selector.notify_controls(cx);
        self.selector.notify_controls(cx);
        self.selection_panel.notify_controls(cx);
        self.selector_templates.notify_controls(cx);
        self.custom_button.notify_controls(cx);
        self.button.notify_controls(cx);
        self.icon_button.notify_controls(cx);
        self.toggle.notify_controls(cx);
        self.toggle_group.notify_controls(cx);
        self.switch.notify_controls(cx);
        self.checkbox.notify_controls(cx);
        self.radio_button.notify_controls(cx);
        self.radio_group.notify_controls(cx);
        self.choice_templates.notify_controls(cx);
        self.listbox.notify_controls(cx);
        self.slider.notify_controls(cx);
        self.scrollbar.notify_controls(cx);
        self.textarea.notify_controls(cx);
        self.textfield.notify_controls(cx);
        self.floating_menu.notify_controls(cx);
        self.popup_menu.notify_controls(cx);
        self.context_menu.notify_controls(cx);
        self.navigation_sidebar.notify_controls(cx);
        self.tabs_navigation.notify_controls(cx);
        self.progress.notify_controls(cx);
    }

    pub(in crate::gallery) fn render_selected(&self, selection: &str) -> AnyElement {
        let Some(page) = page_for_id(selection) else {
            debug_assert!(false, "unknown gallery page id: {selection}");
            return render_unknown_page(selection, &self.theme);
        };

        match page.kind {
            GalleryPageKind::Introduction => self.introduction.render(&self.theme),
            GalleryPageKind::DecoratedButton => self.decorated_button.render(&self.theme),
            GalleryPageKind::AutocompleteTextField => self.autocomplete_textfield.render(&self.theme),
            GalleryPageKind::ComboBox => self.combobox.render(&self.theme),
            GalleryPageKind::SearchSelector => self.search_selector.render(&self.theme),
            GalleryPageKind::Selector => self.selector.render(&self.theme),
            GalleryPageKind::SelectionPanel => self.selection_panel.render(&self.theme),
            GalleryPageKind::SelectorTemplates => self.selector_templates.render(&self.theme),
            GalleryPageKind::Search => search::render(&self.theme),
            GalleryPageKind::Palette => palette::render(&self.theme),
            GalleryPageKind::ThemeUsage => theme_usage::render(&self.theme),
            GalleryPageKind::Button => self.button.render(&self.theme),
            GalleryPageKind::CustomButton => self.custom_button.render(&self.theme),
            GalleryPageKind::IconButton => self.icon_button.render(&self.theme),
            GalleryPageKind::Toggle => self.toggle.render(&self.theme),
            GalleryPageKind::ToggleGroup => self.toggle_group.render(&self.theme),
            GalleryPageKind::Switch => self.switch.render(&self.theme),
            GalleryPageKind::Checkbox => self.checkbox.render(&self.theme),
            GalleryPageKind::RadioButton => self.radio_button.render(&self.theme),
            GalleryPageKind::RadioGroup => self.radio_group.render(&self.theme),
            GalleryPageKind::ChoiceTemplates => self.choice_templates.render(&self.theme),
            GalleryPageKind::ListBox => self.listbox.render(&self.theme),
            GalleryPageKind::Slider => self.slider.render(&self.theme),
            GalleryPageKind::Scrollbar => self.scrollbar.render(&self.theme),
            GalleryPageKind::TextArea => self.textarea.render(&self.theme),
            GalleryPageKind::TextField => self.textfield.render(&self.theme),
            GalleryPageKind::FloatingMenu => self.floating_menu.render(&self.theme),
            GalleryPageKind::PopupMenu => self.popup_menu.render(&self.theme),
            GalleryPageKind::ContextMenu => self.context_menu.render(&self.theme),
            GalleryPageKind::NavigationSidebar => self.navigation_sidebar.render(&self.theme),
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
    let label = page.label;
    let button = Button::new(page.id)
        .typed(page.id == INTRODUCTION_PAGE.id)
        .content(move |_, _| div().child(label).into_any_element())
        .template(sidebar_leaf_template(page.icon, reserve_icon_space, theme))
        .spawn(cx);
    let focus_handle = focus_handle_for(&button, cx);

    route_buttons.push(GalleryRouteButton { page_id: page.id, button: button.clone() });

    let mut node = NavNode::new(page.id).label(page.label).presenter(entity_presenter(button, focus_handle));
    if let Some(icon) = page.icon {
        node = node.icon(icon);
    }

    node
}

fn focus_handle_for<T: Focusable>(entity: &Entity<T>, cx: &mut Context<GalleryApp>) -> FocusHandle {
    entity.read(cx).focus_handle(cx)
}

fn controls_label_presenter(
    _: &NavNodeState,
    _: &mut Window,
    _: &mut App,
    theme: Arc<dyn NavigationSidebarTheme>,
) -> NavHostedContent {
    let appearance = theme.resolve_section();

    NavHostedContent {
        element: div()
            .min_h(gpui::px(appearance.height))
            .pt(gpui::px(8.0))
            .text_size(gpui::px(appearance.typography.size))
            .line_height(gpui::px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1)])))
            .text_color(appearance.label_color)
            .child("Controls")
            .into_any_element(),
        focus_handle: None,
    }
}

struct SidebarDisclosureTemplate {
    icon: LucideIcon,
    theme: Arc<dyn NavigationSidebarTheme>,
}

impl ButtonTemplate<bool> for SidebarDisclosureTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve_branch(model.state, model.size);
        let disclosure_icon = if model.data {
            LucideIcon::ChevronDown
        } else {
            LucideIcon::ChevronRight
        };
        let mut root = div()
            .id(model.id.clone())
            .w_full()
            .min_h(px(appearance.height))
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .px(px(appearance.padding_x))
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .text_color(appearance.foreground)
            .font_weight(appearance.typography.weight)
            .child(render_lucide_icon(self.icon, appearance.icon_color, appearance.icon_size))
            .child(div().flex_1().child((model.content)(model, cx)))
            .child(render_lucide_icon(disclosure_icon, appearance.icon_color, appearance.icon_size));

        if let Some(background) = appearance.background {
            root = root.bg(background);
        }

        if !model.state.disabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            root = root.border_1().border_color(focus_ring);
        }

        root
    }
}

fn sidebar_disclosure_template(icon: LucideIcon, theme: &GalleryThemePack) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(SidebarDisclosureTemplate { icon, theme: theme.navigation_sidebar_theme() })
}

struct SidebarLeafTemplate {
    icon: Option<LucideIcon>,
    reserve_icon_space: bool,
    theme: Arc<dyn NavigationSidebarTheme>,
}

impl ButtonTemplate<bool> for SidebarLeafTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve_item(model.data, model.state, model.size);
        let padding_left = if self.reserve_icon_space {
            appearance.padding_x + appearance.icon_size + appearance.gap
        } else {
            appearance.padding_x
        };
        let placeholder_size = if self.reserve_icon_space {
            appearance.icon_size
        } else {
            0.0
        };
        let mut row = div()
            .id(model.id.clone())
            .w_full()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .min_h(px(appearance.height))
            .pl(px(padding_left))
            .pr(px(appearance.padding_x))
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .text_color(appearance.foreground)
            .child(match self.icon {
                Some(icon) => render_lucide_icon(icon, appearance.icon_color, appearance.icon_size),
                None => div().size(px(placeholder_size)).into_any_element(),
            })
            .child(div().flex_1().child((model.content)(model, cx)));

        if let Some(background) = appearance.background {
            row = row.bg(background);
        }

        if !model.state.disabled {
            row = row.cursor_pointer();
        } else {
            row = row.opacity(0.56);
        }

        if let Some(focus_ring) = appearance.focus_ring {
            row = row.border_1().border_color(focus_ring);
        }

        row
    }
}

fn sidebar_leaf_template(
    icon: Option<LucideIcon>,
    reserve_icon_space: bool,
    theme: &GalleryThemePack,
) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(SidebarLeafTemplate { icon, reserve_icon_space, theme: theme.navigation_sidebar_theme() })
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
