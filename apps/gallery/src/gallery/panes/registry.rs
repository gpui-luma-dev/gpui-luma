use gpui::{AnyElement, Context, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::nav_view::{NavButton, NavItem, NavNodeItem};

use crate::gallery::control::GalleryApp;

use super::{
    button, checkbox, context_menu, dropdown_menu, icon_button, introduction, progress, radio_group, scrollbar, search,
    settings, shared::gallery_pane, slider, switch, toggle_button,
};

#[derive(Clone, Copy)]
struct GalleryPage {
    id: &'static str,
    label: &'static str,
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
    expanded: bool,
    pages: &'static [GalleryPage],
}

const INTRODUCTION_PAGE: GalleryPage =
    GalleryPage { id: "introduction", label: "Introduction", kind: GalleryPageKind::Introduction };
const SEARCH_PAGE: GalleryPage = GalleryPage { id: "search", label: "Search", kind: GalleryPageKind::Search };
const BUTTON_PAGE: GalleryPage = GalleryPage { id: "button", label: "Button", kind: GalleryPageKind::Button };
const ICON_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "icon-button", label: "Icon Button", kind: GalleryPageKind::IconButton };
const TOGGLE_BUTTON_PAGE: GalleryPage =
    GalleryPage { id: "toggle-button", label: "Toggle Button", kind: GalleryPageKind::ToggleButton };
const SWITCH_PAGE: GalleryPage = GalleryPage { id: "switch", label: "Switch", kind: GalleryPageKind::Switch };
const CHECKBOX_PAGE: GalleryPage = GalleryPage { id: "checkbox", label: "Checkbox", kind: GalleryPageKind::Checkbox };
const RADIO_GROUP_PAGE: GalleryPage =
    GalleryPage { id: "radio-group", label: "Radio Group", kind: GalleryPageKind::RadioGroup };
const SLIDER_PAGE: GalleryPage = GalleryPage { id: "slider", label: "Slider", kind: GalleryPageKind::Slider };
const SCROLLBAR_PAGE: GalleryPage =
    GalleryPage { id: "scrollbar", label: "Scrollbar", kind: GalleryPageKind::Scrollbar };
const DROPDOWN_MENU_PAGE: GalleryPage =
    GalleryPage { id: "dropdown-menu", label: "Dropdown Menu", kind: GalleryPageKind::DropdownMenu };
const CONTEXT_MENU_PAGE: GalleryPage =
    GalleryPage { id: "context-menu", label: "Context Menu", kind: GalleryPageKind::ContextMenu };
const PROGRESS_PAGE: GalleryPage = GalleryPage { id: "progress", label: "Progress", kind: GalleryPageKind::Progress };
const SETTINGS_PAGE: GalleryPage = GalleryPage { id: "settings", label: "Settings", kind: GalleryPageKind::Settings };

const PRIMARY_PAGES: &[GalleryPage] = &[INTRODUCTION_PAGE, SEARCH_PAGE];
const BOTTOM_PAGES: &[GalleryPage] = &[SETTINGS_PAGE];
const COMMAND_PAGES: &[GalleryPage] = &[BUTTON_PAGE, ICON_BUTTON_PAGE, TOGGLE_BUTTON_PAGE];
const CHOICE_PAGES: &[GalleryPage] = &[SWITCH_PAGE, CHECKBOX_PAGE, RADIO_GROUP_PAGE];
const INPUT_PAGES: &[GalleryPage] = &[SLIDER_PAGE, SCROLLBAR_PAGE];
const MENU_PAGES: &[GalleryPage] = &[DROPDOWN_MENU_PAGE, CONTEXT_MENU_PAGE];
const FEEDBACK_PAGES: &[GalleryPage] = &[PROGRESS_PAGE];
const CONTROL_GROUPS: &[GalleryNavGroup] = &[
    GalleryNavGroup { id: "command", label: "Command", expanded: true, pages: COMMAND_PAGES },
    GalleryNavGroup { id: "choice", label: "Choice", expanded: true, pages: CHOICE_PAGES },
    GalleryNavGroup { id: "input", label: "Input", expanded: false, pages: INPUT_PAGES },
    GalleryNavGroup { id: "menu", label: "Menu", expanded: false, pages: MENU_PAGES },
    GalleryNavGroup { id: "feedback", label: "Feedback", expanded: false, pages: FEEDBACK_PAGES },
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

    pub(in crate::gallery) fn nav_items() -> Vec<NavItem> {
        let mut items: Vec<NavItem> =
            PRIMARY_PAGES.iter().map(|page| NavItem::button(page.id).label(page.label).into()).collect();

        items.push(NavItem::label("Controls"));
        items.extend(CONTROL_GROUPS.iter().map(|group| {
            NavItem::node(group.id)
                .label(group.label)
                .expanded(group.expanded)
                .children(group.pages.iter().map(|page| NavNodeItem::new(page.id).label(page.label)))
                .into()
        }));
        items
    }

    pub(in crate::gallery) fn bottom_nav_items() -> Vec<NavButton> {
        BOTTOM_PAGES.iter().map(|page| NavButton::new(page.id).label(page.label)).collect()
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

fn render_unknown_page(selection: &str) -> AnyElement {
    gallery_pane(
        "Unknown Page",
        div()
            .text_color(rgb(0x334155))
            .child(format!("No gallery pane is registered for `{selection}`."))
            .into_any_element(),
    )
}
