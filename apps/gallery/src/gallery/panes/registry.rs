use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontFeatures, FontWeight, IntoElement, Stateful,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::navigation_sidebar::{NavHostedContent, NavNode, NavNodeState, entity_presenter};
use gpui_luma::controls::command::button::{Button, ButtonRenderModel, ButtonTemplate, HasPresenter};
use gpui_luma::controls::navigation_sidebar::NavigationSidebarTheme;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::{
    accordion, autocomplete, badge, button, card, checkbox, choice_controls_template, combobox, context_menu,
    floating_menu, tree_view, introduction, list_view, listbox, navigation_sidebar, palette, popup_menu, progress,
    prototypes, radio_button, radio_group, resizable_panels, scrollbar, search, search_selector, selection_panel,
    split_view, selector, selector_controls_template, settings, shared::gallery_pane, slider, switch, tabs_navigation,
    textarea, textfield, theme_usage, toggle, toggle_group,
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
    Badge,
    Card,
    Button,
    DecoratedButton,
    ShadowButton,
    AutocompleteTextField,
    ComboBox,
    SearchSelector,
    Selector,
    SelectionPanel,
    SelectorTemplates,
    CustomButton,
    Toggle,
    ToggleGroup,
    Switch,
    Checkbox,
    Accordion,
    TreeView,
    RadioButton,
    RadioGroup,
    ChoiceTemplates,
    ListBox,
    ScrollingListView,
    PagingListView,
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
    ResizablePanels,
    SplitViewUnified,
    SplitViewInset,
    SplitViewIconRail,
    SplitViewDetached,
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
const BADGE_PAGE: GalleryPage = GalleryPage { id: "badge", label: "Badge", icon: None, kind: GalleryPageKind::Badge };
const CARD_PAGE: GalleryPage = GalleryPage { id: "card", label: "Card", icon: None, kind: GalleryPageKind::Card };
const BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "button", label: "Button", icon: None, kind: GalleryPageKind::Button };
const TOGGLE_PAGE: GalleryPage =
    GalleryPage { id: "toggle", label: "Toggle", icon: None, kind: GalleryPageKind::Toggle };
const TOGGLE_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "toggle-group", label: "Toggle Group", icon: None, kind: GalleryPageKind::ToggleGroup };
const SWITCH_PAGE: GalleryPage =
    GalleryPage { id: "switch", label: "Switch", icon: None, kind: GalleryPageKind::Switch };
const CHECKBOX_PAGE: GalleryPage =
    GalleryPage { id: "checkbox", label: "Checkbox", icon: None, kind: GalleryPageKind::Checkbox };
const ACCORDION_PAGE: GalleryPage =
    GalleryPage { id: "accordion", label: "Accordion", icon: None, kind: GalleryPageKind::Accordion };
const TREE_VIEW_PAGE: GalleryPage =
    GalleryPage { id: "tree-view", label: "Tree View", icon: None, kind: GalleryPageKind::TreeView };
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
const SCROLLING_LIST_VIEW_PAGE: GalleryPage = GalleryPage {
    id: "scrolling-list-view",
    label: "Scrolling List View",
    icon: None,
    kind: GalleryPageKind::ScrollingListView,
};
const PAGING_LIST_VIEW_PAGE: GalleryPage = GalleryPage {
    id: "paging-list-view",
    label: "Paging List View",
    icon: None,
    kind: GalleryPageKind::PagingListView,
};
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
const RESIZABLE_PANELS_PAGE: GalleryPage = GalleryPage {
    id: "resizable-panels",
    label: "Resizable Panels",
    icon: None,
    kind: GalleryPageKind::ResizablePanels,
};
const SPLIT_VIEW_UNIFIED_PAGE: GalleryPage = GalleryPage {
    id: "split-view-unified",
    label: "Split View: Unified",
    icon: None,
    kind: GalleryPageKind::SplitViewUnified,
};
const SPLIT_VIEW_INSET_PAGE: GalleryPage = GalleryPage {
    id: "split-view-inset",
    label: "Split View: Inset",
    icon: None,
    kind: GalleryPageKind::SplitViewInset,
};
const SPLIT_VIEW_ICON_RAIL_PAGE: GalleryPage = GalleryPage {
    id: "split-view-icon-rail",
    label: "Split View: Icon Rail",
    icon: None,
    kind: GalleryPageKind::SplitViewIconRail,
};
const SPLIT_VIEW_DETACHED_PAGE: GalleryPage = GalleryPage {
    id: "split-view-detached",
    label: "Split View: Detached",
    icon: None,
    kind: GalleryPageKind::SplitViewDetached,
};
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
const SHADOW_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "shadow-button", label: "Shadow Button", icon: None, kind: GalleryPageKind::ShadowButton };
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

const PRIMARY_PAGES: &[GalleryPage] = &[INTRODUCTION_PAGE, PALETTE_PAGE, SEARCH_PAGE, THEME_USAGE_PAGE];
const BOTTOM_PAGES: &[GalleryPage] = &[SETTINGS_PAGE];
const COMMAND_PAGES: &[GalleryPage] = &[BUTTON_PAGE, CUSTOM_BUTTON_PAGE];
const CHOICE_PAGES: &[GalleryPage] = &[
    ACCORDION_PAGE,
    CHECKBOX_PAGE,
    CHOICE_TEMPLATES_PAGE,
    LISTBOX_PAGE,
    PAGING_LIST_VIEW_PAGE,
    RADIO_BUTTON_PAGE,
    RADIO_GROUP_PAGE,
    SCROLLING_LIST_VIEW_PAGE,
    SWITCH_PAGE,
    TOGGLE_PAGE,
    TOGGLE_GROUP_PAGE,
    TREE_VIEW_PAGE,
];
const INPUT_PAGES: &[GalleryPage] = &[SCROLLBAR_PAGE, SLIDER_PAGE, TEXTAREA_PAGE, TEXTFIELD_PAGE];
const MENU_PAGES: &[GalleryPage] = &[CONTEXT_MENU_PAGE, FLOATING_MENU_PAGE, POPUP_MENU_PAGE];
const LAYOUT_PAGES: &[GalleryPage] = &[
    CARD_PAGE,
    RESIZABLE_PANELS_PAGE,
    SPLIT_VIEW_DETACHED_PAGE,
    SPLIT_VIEW_ICON_RAIL_PAGE,
    SPLIT_VIEW_INSET_PAGE,
    SPLIT_VIEW_UNIFIED_PAGE,
];
const NAVIGATION_PAGES: &[GalleryPage] = &[NAVIGATION_SIDEBAR_PAGE, TABS_NAVIGATION_PAGE];
const FEEDBACK_PAGES: &[GalleryPage] = &[BADGE_PAGE, PROGRESS_PAGE];
const SELECTION_PAGES: &[GalleryPage] = &[
    AUTOCOMPLETE_TEXTFIELD_PAGE,
    COMBOBOX_PAGE,
    SEARCH_SELECTOR_PAGE,
    SELECTOR_PAGE,
    SELECTION_PANEL_PAGE,
    SELECTOR_TEMPLATES_PAGE,
];
const PROTOTYPES_PAGES: &[GalleryPage] = &[DECORATED_BUTTON_PAGE, SHADOW_BUTTON_PAGE];
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
    GalleryNavGroup { id: "layout", label: "Layout", icon: LucideIcon::Columns2, expanded: false, pages: LAYOUT_PAGES },
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
    pub(in crate::gallery) look: Arc<ShadcnLook>,
    pub(super) introduction: introduction::IntroductionPane,
    pub(super) badge: badge::BadgePane,
    pub(super) card: card::CardPane,
    pub(super) decorated_button: prototypes::ButtonPane,
    pub(super) shadow_button: prototypes::ShadowButtonPane,
    pub(super) autocomplete_textfield: autocomplete::AutocompleteTextFieldPane,
    pub(super) combobox: combobox::ComboBoxPane,
    pub(super) search_selector: search_selector::SearchSelectorPane,
    pub(super) selector: selector::SelectorPane,
    pub(super) selection_panel: selection_panel::SelectionPanelPane,
    pub(super) selector_templates: selector_controls_template::SelectorControlsTemplatePane,
    pub(super) custom_button: prototypes::ModButtonPane,
    pub(super) button: button::ButtonPane,
    pub(super) toggle: toggle::TogglePane,
    pub(super) toggle_group: toggle_group::ToggleGroupPane,
    pub(super) switch: switch::SwitchPane,
    pub(super) checkbox: checkbox::CheckboxPane,
    pub(super) accordion: accordion::AccordionPane,
    pub(super) tree_view: tree_view::TreeViewPane,
    pub(super) radio_button: radio_button::RadioButtonPane,
    pub(super) radio_group: radio_group::RadioGroupPane,
    pub(super) choice_templates: choice_controls_template::ChoiceControlsTemplatePane,
    pub(super) listbox: listbox::ListBoxPane,
    pub(super) scrolling_list_view: list_view::ScrollingListViewPane,
    pub(super) paging_list_view: list_view::PagingListViewPane,
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
    pub(super) resizable_panels: resizable_panels::ResizablePanelsPane,
    pub(super) split_view: split_view::SplitViewPane,
}

impl GalleryPanes {
    pub(in crate::gallery) fn initial_selection() -> &'static str {
        INTRODUCTION_PAGE.id
    }

    pub(in crate::gallery) fn navigation(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> GalleryNavigation {
        let sidebar_theme = look.navigation_sidebar_theme();
        let mut route_buttons = Vec::new();
        let mut branch_buttons = Vec::new();
        let mut nodes: Vec<NavNode> = PRIMARY_PAGES
            .iter()
            .map(|page| nav_node_for_page(page, cx, &mut route_buttons, sidebar_theme.clone()))
            .collect();

        let label_theme = sidebar_theme.clone();
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
                .template(sidebar_disclosure_template(group.icon, sidebar_theme.clone()))
                .spawn(cx);
            let focus_handle = focus_handle_for(&button, cx);
            branch_buttons.push(GalleryBranchButton { node_id: group.id, button: button.clone() });

            NavNode::new(group.id)
                .label(group.label)
                .icon(group.icon)
                .presenter(entity_presenter(button, focus_handle))
                .expanded(group.expanded)
                .children(
                    group
                        .pages
                        .iter()
                        .map(|page| nav_node_for_page(page, cx, &mut route_buttons, sidebar_theme.clone())),
                )
        }));

        let footer_nodes = BOTTOM_PAGES
            .iter()
            .map(|page| nav_node_for_page(page, cx, &mut route_buttons, sidebar_theme.clone()))
            .collect();

        GalleryNavigation { nodes, footer_nodes, route_buttons, branch_buttons }
    }

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        Self {
            look: look.clone(),
            introduction: introduction::IntroductionPane::new(cx, look.clone()),
            badge: badge::BadgePane::new(cx, look.clone()),
            card: card::CardPane::new(cx, look.clone()),
            decorated_button: prototypes::ButtonPane::new(cx, look.clone()),
            shadow_button: prototypes::ShadowButtonPane::new(cx, look.clone()),
            autocomplete_textfield: autocomplete::AutocompleteTextFieldPane::new(cx, look.clone()),
            combobox: combobox::ComboBoxPane::new(cx, look.clone()),
            search_selector: search_selector::SearchSelectorPane::new(cx, look.clone()),
            selector: selector::SelectorPane::new(cx, look.clone()),
            selection_panel: selection_panel::SelectionPanelPane::new(cx, look.clone()),
            selector_templates: selector_controls_template::SelectorControlsTemplatePane::new(cx, look.clone()),
            custom_button: prototypes::ModButtonPane::new(cx, look.clone()),
            button: button::ButtonPane::new(cx, look.clone()),
            toggle: toggle::TogglePane::new(cx, look.clone()),
            toggle_group: toggle_group::ToggleGroupPane::new(cx, look.clone()),
            switch: switch::SwitchPane::new(cx, look.clone()),
            checkbox: checkbox::CheckboxPane::new(cx, look.clone()),
            accordion: accordion::AccordionPane::new(cx, look.clone()),
            tree_view: tree_view::TreeViewPane::new(cx, look.clone()),
            radio_button: radio_button::RadioButtonPane::new(cx, look.clone()),
            radio_group: radio_group::RadioGroupPane::new(cx, look.clone()),
            choice_templates: choice_controls_template::ChoiceControlsTemplatePane::new(cx, look.clone()),
            listbox: listbox::ListBoxPane::new(cx, look.clone()),
            scrolling_list_view: list_view::ScrollingListViewPane::new(cx, look.clone()),
            paging_list_view: list_view::PagingListViewPane::new(cx, look.clone()),
            slider: slider::SliderPane::new(cx, look.clone()),
            scrollbar: scrollbar::ScrollbarPane::new(cx, look.clone()),
            textarea: textarea::TextAreaPane::new(cx, look.clone()),
            textfield: textfield::TextFieldPane::new(cx, look.clone()),
            floating_menu: floating_menu::FloatingMenuPane::new(cx, look.clone()),
            popup_menu: popup_menu::PopupMenuPane::new(cx, look.clone()),
            context_menu: context_menu::ContextMenuPane::new(cx, look.clone()),
            navigation_sidebar: navigation_sidebar::NavigationSidebarPane::new(cx, look.clone()),
            tabs_navigation: tabs_navigation::TabsNavigationPane::new(cx, look.clone()),
            progress: progress::ProgressPane::new(cx, look.clone()),
            resizable_panels: resizable_panels::ResizablePanelsPane::new(cx, look.clone()),
            split_view: split_view::SplitViewPane::new(cx, look.clone()),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.introduction.subscribe(cx, subscriptions);
        self.badge.subscribe(cx, subscriptions);
        self.card.subscribe(cx, subscriptions);
        self.decorated_button.subscribe(cx, subscriptions);
        self.shadow_button.subscribe(cx, subscriptions);
        self.autocomplete_textfield.subscribe(cx, subscriptions);
        self.combobox.subscribe(cx, subscriptions);
        self.search_selector.subscribe(cx, subscriptions);
        self.selector.subscribe(cx, subscriptions);
        self.selection_panel.subscribe(cx, subscriptions);
        self.custom_button.subscribe(cx, subscriptions);
        self.button.subscribe(cx, subscriptions);
        self.toggle.subscribe(cx, subscriptions);
        self.toggle_group.subscribe(cx, subscriptions);
        self.switch.subscribe(cx, subscriptions);
        self.checkbox.subscribe(cx, subscriptions);
        self.accordion.subscribe(cx, subscriptions);
        self.tree_view.subscribe(cx, subscriptions);
        self.radio_button.subscribe(cx, subscriptions);
        self.radio_group.subscribe(cx, subscriptions);
        self.listbox.subscribe(cx, subscriptions);
        self.scrolling_list_view.subscribe(cx, subscriptions);
        self.paging_list_view.subscribe(cx, subscriptions);
        self.slider.subscribe(cx, subscriptions);
        self.scrollbar.subscribe(cx, subscriptions);
        self.textarea.subscribe(cx, subscriptions);
        self.textfield.subscribe(cx, subscriptions);
        self.floating_menu.subscribe(cx, subscriptions);
        self.popup_menu.subscribe(cx, subscriptions);
        self.context_menu.subscribe(cx, subscriptions);
        self.navigation_sidebar.subscribe(cx, subscriptions);
        self.tabs_navigation.subscribe(cx, subscriptions);
        self.resizable_panels.subscribe(cx, subscriptions);
        self.split_view.subscribe(cx, subscriptions);
    }

    pub(in crate::gallery) fn notify_selected_controls(&self, selection: &str, cx: &mut Context<GalleryApp>) {
        let Some(page) = page_for_id(selection) else {
            return;
        };

        match page.kind {
            GalleryPageKind::Introduction => self.introduction.notify_controls(cx),
            GalleryPageKind::Badge => self.badge.notify_controls(cx),
            GalleryPageKind::Card => self.card.notify_controls(cx),
            GalleryPageKind::DecoratedButton => self.decorated_button.notify_controls(cx),
            GalleryPageKind::ShadowButton => self.shadow_button.notify_controls(cx),
            GalleryPageKind::AutocompleteTextField => self.autocomplete_textfield.notify_controls(cx),
            GalleryPageKind::ComboBox => self.combobox.notify_controls(cx),
            GalleryPageKind::SearchSelector => self.search_selector.notify_controls(cx),
            GalleryPageKind::Selector => self.selector.notify_controls(cx),
            GalleryPageKind::SelectionPanel => self.selection_panel.notify_controls(cx),
            GalleryPageKind::SelectorTemplates => self.selector_templates.notify_controls(cx),
            GalleryPageKind::CustomButton => self.custom_button.notify_controls(cx),
            GalleryPageKind::Button => self.button.notify_controls(cx),
            GalleryPageKind::Toggle => self.toggle.notify_controls(cx),
            GalleryPageKind::ToggleGroup => self.toggle_group.notify_controls(cx),
            GalleryPageKind::Switch => self.switch.notify_controls(cx),
            GalleryPageKind::Checkbox => self.checkbox.notify_controls(cx),
            GalleryPageKind::Accordion => self.accordion.notify_controls(cx),
            GalleryPageKind::TreeView => self.tree_view.notify_controls(cx),
            GalleryPageKind::RadioButton => self.radio_button.notify_controls(cx),
            GalleryPageKind::RadioGroup => self.radio_group.notify_controls(cx),
            GalleryPageKind::ChoiceTemplates => self.choice_templates.notify_controls(cx),
            GalleryPageKind::ListBox => self.listbox.notify_controls(cx),
            GalleryPageKind::ScrollingListView => self.scrolling_list_view.notify_controls(cx),
            GalleryPageKind::PagingListView => self.paging_list_view.notify_controls(cx),
            GalleryPageKind::Slider => self.slider.notify_controls(cx),
            GalleryPageKind::Scrollbar => self.scrollbar.notify_controls(cx),
            GalleryPageKind::TextArea => self.textarea.notify_controls(cx),
            GalleryPageKind::TextField => self.textfield.notify_controls(cx),
            GalleryPageKind::FloatingMenu => self.floating_menu.notify_controls(cx),
            GalleryPageKind::PopupMenu => self.popup_menu.notify_controls(cx),
            GalleryPageKind::ContextMenu => self.context_menu.notify_controls(cx),
            GalleryPageKind::NavigationSidebar => self.navigation_sidebar.notify_controls(cx),
            GalleryPageKind::TabsNavigation => self.tabs_navigation.notify_controls(cx),
            GalleryPageKind::Progress => self.progress.notify_controls(cx),
            GalleryPageKind::ResizablePanels => self.resizable_panels.notify_controls(cx),
            GalleryPageKind::SplitViewUnified
            | GalleryPageKind::SplitViewInset
            | GalleryPageKind::SplitViewIconRail
            | GalleryPageKind::SplitViewDetached => self.split_view.notify_controls(cx),
            GalleryPageKind::Search
            | GalleryPageKind::Palette
            | GalleryPageKind::ThemeUsage
            | GalleryPageKind::Settings => {}
        }
    }

    pub(in crate::gallery) fn render_selected(&self, selection: &str) -> AnyElement {
        let Some(page) = page_for_id(selection) else {
            debug_assert!(false, "unknown gallery page id: {selection}");
            return render_unknown_page(selection, &self.look);
        };

        match page.kind {
            GalleryPageKind::Introduction => self.introduction.render(&self.look),
            GalleryPageKind::Badge => self.badge.render(&self.look),
            GalleryPageKind::Card => self.card.render(&self.look),
            GalleryPageKind::DecoratedButton => self.decorated_button.render(&self.look),
            GalleryPageKind::ShadowButton => self.shadow_button.render(&self.look),
            GalleryPageKind::AutocompleteTextField => self.autocomplete_textfield.render(&self.look),
            GalleryPageKind::ComboBox => self.combobox.render(&self.look),
            GalleryPageKind::SearchSelector => self.search_selector.render(&self.look),
            GalleryPageKind::Selector => self.selector.render(&self.look),
            GalleryPageKind::SelectionPanel => self.selection_panel.render(&self.look),
            GalleryPageKind::SelectorTemplates => self.selector_templates.render(&self.look),
            GalleryPageKind::Search => search::render(&self.look),
            GalleryPageKind::Palette => palette::render(&self.look),
            GalleryPageKind::ThemeUsage => theme_usage::render(&self.look),
            GalleryPageKind::Button => self.button.render(&self.look),
            GalleryPageKind::CustomButton => self.custom_button.render(&self.look),
            GalleryPageKind::Toggle => self.toggle.render(&self.look),
            GalleryPageKind::ToggleGroup => self.toggle_group.render(&self.look),
            GalleryPageKind::Switch => self.switch.render(&self.look),
            GalleryPageKind::Checkbox => self.checkbox.render(&self.look),
            GalleryPageKind::Accordion => self.accordion.render(&self.look),
            GalleryPageKind::TreeView => self.tree_view.render(&self.look),
            GalleryPageKind::RadioButton => self.radio_button.render(&self.look),
            GalleryPageKind::RadioGroup => self.radio_group.render(&self.look),
            GalleryPageKind::ChoiceTemplates => self.choice_templates.render(&self.look),
            GalleryPageKind::ListBox => self.listbox.render(&self.look),
            GalleryPageKind::ScrollingListView => self.scrolling_list_view.render(&self.look),
            GalleryPageKind::PagingListView => self.paging_list_view.render(&self.look),
            GalleryPageKind::Slider => self.slider.render(&self.look),
            GalleryPageKind::Scrollbar => self.scrollbar.render(&self.look),
            GalleryPageKind::TextArea => self.textarea.render(&self.look),
            GalleryPageKind::TextField => self.textfield.render(&self.look),
            GalleryPageKind::FloatingMenu => self.floating_menu.render(&self.look),
            GalleryPageKind::PopupMenu => self.popup_menu.render(&self.look),
            GalleryPageKind::ContextMenu => self.context_menu.render(&self.look),
            GalleryPageKind::NavigationSidebar => self.navigation_sidebar.render(&self.look),
            GalleryPageKind::TabsNavigation => self.tabs_navigation.render(&self.look),
            GalleryPageKind::Progress => self.progress.render(&self.look),
            GalleryPageKind::ResizablePanels => self.resizable_panels.render(&self.look),
            GalleryPageKind::SplitViewUnified => {
                self.split_view.render(split_view::SplitViewDemoKind::Unified, &self.look)
            }
            GalleryPageKind::SplitViewInset => self.split_view.render(split_view::SplitViewDemoKind::Inset, &self.look),
            GalleryPageKind::SplitViewIconRail => {
                self.split_view.render(split_view::SplitViewDemoKind::IconRail, &self.look)
            }
            GalleryPageKind::SplitViewDetached => {
                self.split_view.render(split_view::SplitViewDemoKind::Detached, &self.look)
            }
            GalleryPageKind::Settings => settings::render(&self.look),
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
    sidebar_theme: Arc<dyn NavigationSidebarTheme>,
) -> NavNode {
    let reserve_icon_space = PRIMARY_PAGES.iter().chain(BOTTOM_PAGES).all(|candidate| candidate.id != page.id);
    let label = page.label;
    let button = Button::new(page.id)
        .typed(page.id == INTRODUCTION_PAGE.id)
        .content(move |_, _| div().child(label).into_any_element())
        .template(sidebar_leaf_template(page.icon, reserve_icon_space, sidebar_theme.clone()))
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

fn sidebar_disclosure_template(
    icon: LucideIcon,
    theme: Arc<dyn NavigationSidebarTheme>,
) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(SidebarDisclosureTemplate { icon, theme })
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
    theme: Arc<dyn NavigationSidebarTheme>,
) -> Arc<dyn ButtonTemplate<bool>> {
    Arc::new(SidebarLeafTemplate { icon, reserve_icon_space, theme })
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

fn render_unknown_page(selection: &str, look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();

    gallery_pane(
        "Unknown Page",
        div()
            .text_color(chrome.body_text)
            .child(format!("No gallery pane is registered for `{selection}`."))
            .into_any_element(),
        look,
    )
}
