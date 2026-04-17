use std::sync::Arc;

use gpui::{Context, Entity, Hsla, IntoElement, Render, Subscription, Window, div, prelude::*, px, rgb};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent, CheckboxTemplate, ThemedCheckboxTemplate};
use gpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use gpui_luma::controls::dropdown_menu::{DropdownMenu, DropdownMenuEvent, DropdownMenuItem};
use gpui_luma::controls::icon_button::{IconButton, IconButtonEvent, IconButtonKind};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonEvent};
use gpui_luma::theme::{CheckboxAppearance, CheckboxTheme, DefaultCheckboxTheme, InteractionState};
use lucide_icons::Icon as LucideIcon;

mod radial_context_menu;

use radial_context_menu::radial_context_menu_template;

struct GalleryCheckboxTheme {
    base: DefaultCheckboxTheme,
    control_border: Option<Hsla>,
    control_background: Option<Hsla>,
}

impl GalleryCheckboxTheme {
    fn new(control_border: Option<Hsla>, control_background: Option<Hsla>) -> Self {
        Self {
            base: DefaultCheckboxTheme::default(),
            control_border,
            control_background,
        }
    }
}

impl CheckboxTheme for GalleryCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let mut appearance = self.base.resolve(checked, state);

        if !state.disabled {
            if let Some(control_border) = self.control_border {
                appearance.control_border = Some(control_border);
                appearance.control_padding_x = 10.0;
                appearance.control_padding_y = 6.0;
            }

            if let Some(control_background) = self.control_background {
                appearance.control_background = Some(control_background);
            }
        }

        appearance
    }
}

#[derive(Clone, Copy)]
enum CheckboxPresentation {
    Default,
    Border,
    BorderAndBackground,
}

#[derive(Clone, Copy)]
enum ContextMenuPresentation {
    Default,
    Radial,
}

pub struct GalleryApp {
    button: Entity<Button>,
    icon_button: Entity<IconButton>,
    toggle_button: Entity<ToggleButton>,
    default_checkbox: Entity<Checkbox>,
    border_checkbox: Entity<Checkbox>,
    filled_checkbox: Entity<Checkbox>,
    dropdown_menu: Entity<DropdownMenu>,
    default_context_menu: Entity<ContextMenu>,
    radial_context_menu: Entity<ContextMenu>,
    disabled_button: Entity<Button>,
    disabled_icon_button: Entity<IconButton>,
    disabled_toggle_button: Entity<ToggleButton>,
    disabled_checkbox: Entity<Checkbox>,
    clicks: usize,
    icon_clicks: usize,
    toggle_selected: bool,
    default_checkbox_checked: bool,
    border_checkbox_checked: bool,
    filled_checkbox_checked: bool,
    dropdown_selection: String,
    default_context_selection: String,
    radial_context_selection: String,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let border_checkbox_template = checkbox_template(Arc::new(GalleryCheckboxTheme::new(
            Some(rgb(0x2563eb).into()),
            None,
        )));
        let filled_checkbox_template = checkbox_template(Arc::new(GalleryCheckboxTheme::new(
            Some(rgb(0xbe185d).into()),
            Some(rgb(0xfce7f3).into()),
        )));

        let button = Button::new("button-example")
            .label("Click me")
            .kind(ButtonKind::Primary)
            .spawn(cx);
        let icon_button = IconButton::new("icon-button-example", LucideIcon::Plus)
            .kind(IconButtonKind::Primary)
            .spawn(cx);
        let toggle_button = ToggleButton::new("toggle-button-example")
            .label("Toggle")
            .selected(true)
            .spawn(cx);
        let default_checkbox = Checkbox::new("checkbox-default")
            .label("As-is")
            .checked(true)
            .spawn(cx);
        let border_checkbox = Checkbox::new("checkbox-border")
            .label("Border")
            .template(border_checkbox_template)
            .spawn(cx);
        let filled_checkbox = Checkbox::new("checkbox-border-background")
            .label("Border + fill")
            .checked(true)
            .template(filled_checkbox_template)
            .spawn(cx);
        let dropdown_menu = DropdownMenu::new("dropdown-menu-example")
            .label("Actions")
            .items([
                DropdownMenuItem::new("new")
                    .label("New file")
                    .icon(LucideIcon::FilePlus),
                DropdownMenuItem::new("rename")
                    .label("Rename")
                    .icon(LucideIcon::Pencil),
                DropdownMenuItem::new("archive").label("Archive"),
                DropdownMenuItem::new("share")
                    .label("Share")
                    .icon(LucideIcon::Share2)
                    .submenu([
                        DropdownMenuItem::new("copy-link")
                            .label("Copy link")
                            .icon(LucideIcon::Link),
                        DropdownMenuItem::new("email")
                            .label("Email")
                            .icon(LucideIcon::Mail),
                    ]),
                DropdownMenuItem::new("disabled")
                    .label("Unavailable")
                    .icon(LucideIcon::ArchiveX)
                    .enabled(false),
            ])
            .spawn(cx);
        let default_context_menu = ContextMenu::new("context-menu-default-example")
            .label("Right-click target")
            .items(default_context_menu_items())
            .spawn(cx);
        let radial_context_menu = ContextMenu::new("context-menu-radial-example")
            .label("Radial context target")
            .items([
                DropdownMenuItem::new("open")
                    .label("Open")
                    .icon(LucideIcon::FolderOpen),
                DropdownMenuItem::new("copy")
                    .label("Copy")
                    .icon(LucideIcon::Copy),
                DropdownMenuItem::new("inspect")
                    .label("Inspect")
                    .icon(LucideIcon::ScanSearch),
                DropdownMenuItem::new("download")
                    .label("Download")
                    .icon(LucideIcon::Download),
                DropdownMenuItem::new("external")
                    .label("Open externally")
                    .icon(LucideIcon::ExternalLink),
            ])
            .template(radial_context_menu_template())
            .spawn(cx);
        let disabled_button = Button::new("disabled-button")
            .label("Disabled")
            .enabled(false)
            .spawn(cx);
        let disabled_icon_button = IconButton::new("disabled-icon-button", LucideIcon::Check)
            .enabled(false)
            .spawn(cx);
        let disabled_toggle_button = ToggleButton::new("disabled-toggle-button")
            .label("Disabled toggle")
            .selected(true)
            .enabled(false)
            .spawn(cx);
        let disabled_checkbox = Checkbox::new("disabled-checkbox")
            .label("Disabled checkbox")
            .checked(true)
            .enabled(false)
            .spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&button, |this, _, event: &ButtonEvent, cx| {
                this.handle_button_event(event, cx);
            }),
            cx.subscribe(&icon_button, |this, _, event: &IconButtonEvent, cx| {
                this.handle_icon_button_event(event, cx);
            }),
            cx.subscribe(&toggle_button, |this, _, event: &ToggleButtonEvent, cx| {
                this.handle_toggle_button_event(event, cx);
            }),
            cx.subscribe(&default_checkbox, |this, _, event: &CheckboxEvent, cx| {
                this.handle_checkbox_event(CheckboxPresentation::Default, event, cx);
            }),
            cx.subscribe(&border_checkbox, |this, _, event: &CheckboxEvent, cx| {
                this.handle_checkbox_event(CheckboxPresentation::Border, event, cx);
            }),
            cx.subscribe(&filled_checkbox, |this, _, event: &CheckboxEvent, cx| {
                this.handle_checkbox_event(CheckboxPresentation::BorderAndBackground, event, cx);
            }),
            cx.subscribe(&dropdown_menu, |this, _, event: &DropdownMenuEvent, cx| {
                this.handle_dropdown_menu_event(event, cx);
            }),
            cx.subscribe(
                &default_context_menu,
                |this, _, event: &ContextMenuEvent, cx| {
                    this.handle_context_menu_event(ContextMenuPresentation::Default, event, cx);
                },
            ),
            cx.subscribe(
                &radial_context_menu,
                |this, _, event: &ContextMenuEvent, cx| {
                    this.handle_context_menu_event(ContextMenuPresentation::Radial, event, cx);
                },
            ),
        ];

        Self {
            button,
            icon_button,
            toggle_button,
            default_checkbox,
            border_checkbox,
            filled_checkbox,
            dropdown_menu,
            default_context_menu,
            radial_context_menu,
            disabled_button,
            disabled_icon_button,
            disabled_toggle_button,
            disabled_checkbox,
            clicks: 0,
            icon_clicks: 0,
            toggle_selected: true,
            default_checkbox_checked: true,
            border_checkbox_checked: false,
            filled_checkbox_checked: true,
            dropdown_selection: "none".to_string(),
            default_context_selection: "none".to_string(),
            radial_context_selection: "none".to_string(),
            _subscriptions: subscriptions,
        }
    }

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        match event {
            ButtonEvent::Click => {
                self.clicks += 1;
                let label = format!("Clicked {}", self.clicks);

                self.button.update(cx, |button, cx| {
                    button.set_label(label, cx);
                });
            }
        }
    }

    fn handle_icon_button_event(&mut self, event: &IconButtonEvent, cx: &mut Context<Self>) {
        match event {
            IconButtonEvent::Click => {
                self.icon_clicks += 1;
                let icon = if self.icon_clicks.is_multiple_of(2) {
                    LucideIcon::Plus
                } else {
                    LucideIcon::Check
                };

                self.icon_button.update(cx, |button, cx| {
                    button.set_icon(icon, cx);
                });
            }
        }
    }

    fn handle_toggle_button_event(&mut self, event: &ToggleButtonEvent, cx: &mut Context<Self>) {
        match event {
            ToggleButtonEvent::Change { selected } => {
                self.toggle_selected = *selected;
                cx.notify();
            }
        }
    }

    fn handle_checkbox_event(
        &mut self,
        presentation: CheckboxPresentation,
        event: &CheckboxEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            CheckboxEvent::Change { checked } => {
                match presentation {
                    CheckboxPresentation::Default => {
                        self.default_checkbox_checked = *checked;
                    }
                    CheckboxPresentation::Border => {
                        self.border_checkbox_checked = *checked;
                    }
                    CheckboxPresentation::BorderAndBackground => {
                        self.filled_checkbox_checked = *checked;
                    }
                }
                cx.notify();
            }
        }
    }

    fn handle_dropdown_menu_event(&mut self, event: &DropdownMenuEvent, cx: &mut Context<Self>) {
        match event {
            DropdownMenuEvent::Select { label, .. } => {
                self.dropdown_selection = label.to_string();
                cx.notify();
            }
        }
    }

    fn handle_context_menu_event(
        &mut self,
        presentation: ContextMenuPresentation,
        event: &ContextMenuEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ContextMenuEvent::Select { label, .. } => {
                match presentation {
                    ContextMenuPresentation::Default => {
                        self.default_context_selection = label.to_string();
                    }
                    ContextMenuPresentation::Radial => {
                        self.radial_context_selection = label.to_string();
                    }
                }
                cx.notify();
            }
        }
    }
}

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(rgb(0xf8fafc))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.button.clone())
                    .child(self.icon_button.clone())
                    .child(self.toggle_button.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.default_checkbox.clone())
                    .child(self.border_checkbox.clone())
                    .child(self.filled_checkbox.clone()),
            )
            .child(self.dropdown_menu.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .min_h(px(220.0))
                    .child(self.default_context_menu.clone())
                    .child(
                        div()
                            .min_w(px(260.0))
                            .min_h(px(220.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.radial_context_menu.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .text_color(rgb(0x334155))
                    .child(format!("Toggle selected: {}", self.toggle_selected))
                    .child(format!(
                        "Checkboxes: as-is={}, border={}, border + fill={}",
                        self.default_checkbox_checked,
                        self.border_checkbox_checked,
                        self.filled_checkbox_checked
                    ))
                    .child(format!("Dropdown selected: {}", self.dropdown_selection))
                    .child(format!(
                        "Default context selected: {}",
                        self.default_context_selection
                    ))
                    .child(format!(
                        "Radial context selected: {}",
                        self.radial_context_selection
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.disabled_button.clone())
                    .child(self.disabled_icon_button.clone())
                    .child(self.disabled_toggle_button.clone())
                    .child(self.disabled_checkbox.clone()),
            )
    }
}

fn checkbox_template(theme: Arc<dyn CheckboxTheme>) -> Arc<dyn CheckboxTemplate> {
    Arc::new(ThemedCheckboxTemplate::new(theme))
}

fn default_context_menu_items() -> [DropdownMenuItem; 4] {
    [
        DropdownMenuItem::new("open")
            .label("Open")
            .icon(LucideIcon::FolderOpen),
        DropdownMenuItem::new("copy")
            .label("Copy")
            .icon(LucideIcon::Copy),
        DropdownMenuItem::new("inspect").label("Inspect"),
        DropdownMenuItem::new("more")
            .label("More")
            .icon(LucideIcon::Ellipsis)
            .submenu([
                DropdownMenuItem::new("download")
                    .label("Download")
                    .icon(LucideIcon::Download),
                DropdownMenuItem::new("external")
                    .label("Open externally")
                    .icon(LucideIcon::ExternalLink),
            ]),
    ]
}
