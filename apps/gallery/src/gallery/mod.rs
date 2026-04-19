use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FocusHandle, Hsla, IntoElement, MouseButton, Render, Subscription, Window, div,
    prelude::*, px, rgb,
};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent, CheckboxTemplate, ThemedCheckboxTemplate};
use gpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use gpui_luma::controls::dropdown_menu::{DropdownMenu, DropdownMenuEvent, DropdownMenuItem};
use gpui_luma::controls::icon_button::{IconButton, IconButtonEvent, IconButtonKind};
use gpui_luma::controls::progress::Progress;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem};
use gpui_luma::controls::scrollbar::{Scrollbar, ScrollbarEvent};
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::split_view::{SplitView, SplitViewEvent};
use gpui_luma::controls::switch::{Switch, SwitchEvent};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonEvent};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::theme::{CheckboxAppearance, CheckboxTheme, DefaultCheckboxTheme, InteractionState};
use lucide_icons::Icon as LucideIcon;

mod radial_context_menu;

use radial_context_menu::radial_context_menu_template;

struct GalleryCheckboxTheme {
    base: DefaultCheckboxTheme,
    control_border: Option<Hsla>,
    control_background: Option<Hsla>,
    control_padding: Option<(f32, f32)>,
}

impl GalleryCheckboxTheme {
    fn new() -> Self {
        Self {
            base: DefaultCheckboxTheme::default(),
            control_border: None,
            control_background: None,
            control_padding: None,
        }
    }

    fn control_border(mut self, color: impl Into<Hsla>) -> Self {
        self.control_border = Some(color.into());
        self
    }

    fn control_background(mut self, color: impl Into<Hsla>) -> Self {
        self.control_background = Some(color.into());
        self
    }

    fn control_padding(mut self, x: f32, y: f32) -> Self {
        self.control_padding = Some((x, y));
        self
    }
}

impl CheckboxTheme for GalleryCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let mut appearance = self.base.resolve(checked, state);

        if !state.disabled {
            if let Some(control_border) = self.control_border {
                appearance.control_border = Some(control_border);
            }

            if let Some(control_background) = self.control_background {
                appearance.control_background = Some(control_background);
            }

            if let Some((x, y)) = self.control_padding {
                appearance.control_padding_x = x;
                appearance.control_padding_y = y;
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

#[derive(Clone, Copy)]
enum ScrollbarPresentation {
    Horizontal,
    Vertical,
}

pub struct GalleryApp {
    focus_scope: FocusHandle,
    split_view: Entity<SplitView>,
    button: Entity<Button>,
    icon_button: Entity<IconButton>,
    toggle_button: Entity<ToggleButton>,
    switch: Entity<Switch>,
    default_checkbox: Entity<Checkbox>,
    border_checkbox: Entity<Checkbox>,
    filled_checkbox: Entity<Checkbox>,
    radio_group: Entity<RadioGroup>,
    slider: Entity<Slider>,
    progress: Entity<Progress>,
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    dropdown_menu: Entity<DropdownMenu>,
    default_context_menu: Entity<ContextMenu>,
    radial_context_menu: Entity<ContextMenu>,
    disabled_button: Entity<Button>,
    disabled_icon_button: Entity<IconButton>,
    disabled_toggle_button: Entity<ToggleButton>,
    disabled_switch: Entity<Switch>,
    disabled_checkbox: Entity<Checkbox>,
    clicks: usize,
    icon_clicks: usize,
    toggle_selected: bool,
    switch_on: bool,
    default_checkbox_checked: bool,
    border_checkbox_checked: bool,
    filled_checkbox_checked: bool,
    radio_choice: String,
    slider_value: f32,
    horizontal_scroll_value: f32,
    vertical_scroll_value: f32,
    dropdown_selection: String,
    default_context_selection: String,
    radial_context_selection: String,
    split_sidebar_width: f32,
    split_sidebar_collapsed: bool,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        window.focus(&focus_scope, cx);

        let border_checkbox_template =
            checkbox_template(GalleryCheckboxTheme::new().control_border(rgb(0x2563eb)).control_padding(10.0, 6.0));
        let filled_checkbox_template = checkbox_template(
            GalleryCheckboxTheme::new()
                .control_border(rgb(0xbe185d))
                .control_background(rgb(0xfce7f3))
                .control_padding(10.0, 6.0),
        );

        let split_view = SplitView::new("gallery-shell")
            .sidebar_width(px(280.0))
            .sidebar_min_width(px(220.0))
            .sidebar_max_width(px(420.0))
            .sidebar_collapsed_width(px(0.0))
            .collapsed(false)
            .resizable(true)
            .spawn(cx);
        let button = Button::new("button-example").label("Click me").kind(ButtonKind::Primary).spawn(cx);
        let icon_button =
            IconButton::new("icon-button-example", LucideIcon::Plus).kind(IconButtonKind::Primary).spawn(cx);
        let toggle_button = ToggleButton::new("toggle-button-example").label("Toggle").selected(true).spawn(cx);
        let switch = Switch::new("switch-example").on(true).spawn(cx);
        let default_checkbox = Checkbox::new("checkbox-default").label("As-is").checked(true).spawn(cx);
        let border_checkbox =
            Checkbox::new("checkbox-border").label("Border").template(border_checkbox_template).spawn(cx);
        let filled_checkbox = Checkbox::new("checkbox-border-background")
            .label("Border + fill")
            .checked(true)
            .template(filled_checkbox_template)
            .spawn(cx);
        let radio_group = RadioGroup::new("density-radio-group")
            .items([
                RadioGroupItem::new("compact").label("Compact"),
                RadioGroupItem::new("comfortable").label("Comfortable"),
                RadioGroupItem::new("expanded").label("Expanded"),
            ])
            .selected("comfortable")
            .spawn(cx);
        let slider = Slider::new("slider-example").range(1..100).step(10).value(41).spawn(cx);
        let progress = Progress::new("progress-example").range(1..100).value(41).spawn(cx);
        let horizontal_scrollbar = Scrollbar::new("scrollbar-horizontal-example")
            .horizontal()
            .range(0..220)
            .step(20)
            .page_step(80)
            .value(40)
            .thumb_fraction(0.54)
            .spawn(cx);
        let vertical_scrollbar = Scrollbar::new("scrollbar-vertical-example")
            .vertical()
            .range(0..240)
            .step(20)
            .page_step(80)
            .value(80)
            .thumb_fraction(0.45)
            .spawn(cx);
        let dropdown_menu = DropdownMenu::new("dropdown-menu-example")
            .label("Actions")
            .items([
                DropdownMenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
                DropdownMenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
                DropdownMenuItem::new("archive").label("Archive"),
                DropdownMenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
                    DropdownMenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
                    DropdownMenuItem::new("email").label("Email").icon(LucideIcon::Mail),
                ]),
                DropdownMenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
            ])
            .spawn(cx);
        let default_context_menu = ContextMenu::new("context-menu-default-example")
            .label("Right-click target")
            .items(default_context_menu_items())
            .spawn(cx);
        let radial_context_menu = ContextMenu::new("context-menu-radial-example")
            .label("Radial context target")
            .items([
                DropdownMenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
                DropdownMenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
                DropdownMenuItem::new("inspect").label("Inspect").icon(LucideIcon::ScanSearch),
                DropdownMenuItem::new("download").label("Download").icon(LucideIcon::Download),
                DropdownMenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
            ])
            .template(radial_context_menu_template())
            .spawn(cx);
        let disabled_button = Button::new("disabled-button").label("Disabled").enabled(false).spawn(cx);
        let disabled_icon_button = IconButton::new("disabled-icon-button", LucideIcon::Check).enabled(false).spawn(cx);
        let disabled_toggle_button = ToggleButton::new("disabled-toggle-button")
            .label("Disabled toggle")
            .selected(true)
            .enabled(false)
            .spawn(cx);
        let disabled_switch = Switch::new("disabled-switch").on(true).enabled(false).spawn(cx);
        let disabled_checkbox =
            Checkbox::new("disabled-checkbox").label("Disabled checkbox").checked(true).enabled(false).spawn(cx);

        let subscriptions = vec![
            cx.subscribe(&split_view, |this, _, event: &SplitViewEvent, cx| {
                this.handle_split_view_event(event, cx);
            }),
            cx.subscribe(&button, |this, _, event: &ButtonEvent, cx| {
                this.handle_button_event(event, cx);
            }),
            cx.subscribe(&icon_button, |this, _, event: &IconButtonEvent, cx| {
                this.handle_icon_button_event(event, cx);
            }),
            cx.subscribe(&toggle_button, |this, _, event: &ToggleButtonEvent, cx| {
                this.handle_toggle_button_event(event, cx);
            }),
            cx.subscribe(&switch, |this, _, event: &SwitchEvent, cx| {
                this.handle_switch_event(event, cx);
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
            cx.subscribe(&radio_group, |this, _, event: &RadioGroupEvent, cx| {
                this.handle_radio_group_event(event, cx);
            }),
            cx.subscribe(&slider, |this, _, event: &SliderEvent, cx| {
                this.handle_slider_event(event, cx);
            }),
            cx.subscribe(&horizontal_scrollbar, |this, _, event: &ScrollbarEvent, cx| {
                this.handle_scrollbar_event(ScrollbarPresentation::Horizontal, event, cx);
            }),
            cx.subscribe(&vertical_scrollbar, |this, _, event: &ScrollbarEvent, cx| {
                this.handle_scrollbar_event(ScrollbarPresentation::Vertical, event, cx);
            }),
            cx.subscribe(&dropdown_menu, |this, _, event: &DropdownMenuEvent, cx| {
                this.handle_dropdown_menu_event(event, cx);
            }),
            cx.subscribe(&default_context_menu, |this, _, event: &ContextMenuEvent, cx| {
                this.handle_context_menu_event(ContextMenuPresentation::Default, event, cx);
            }),
            cx.subscribe(&radial_context_menu, |this, _, event: &ContextMenuEvent, cx| {
                this.handle_context_menu_event(ContextMenuPresentation::Radial, event, cx);
            }),
        ];

        Self {
            focus_scope,
            split_view,
            button,
            icon_button,
            toggle_button,
            switch,
            default_checkbox,
            border_checkbox,
            filled_checkbox,
            radio_group,
            slider,
            progress,
            horizontal_scrollbar,
            vertical_scrollbar,
            dropdown_menu,
            default_context_menu,
            radial_context_menu,
            disabled_button,
            disabled_icon_button,
            disabled_toggle_button,
            disabled_switch,
            disabled_checkbox,
            clicks: 0,
            icon_clicks: 0,
            toggle_selected: true,
            switch_on: true,
            default_checkbox_checked: true,
            border_checkbox_checked: false,
            filled_checkbox_checked: true,
            radio_choice: "Comfortable".to_string(),
            slider_value: 41.0,
            horizontal_scroll_value: 40.0,
            vertical_scroll_value: 80.0,
            dropdown_selection: "none".to_string(),
            default_context_selection: "none".to_string(),
            radial_context_selection: "none".to_string(),
            split_sidebar_width: 280.0,
            split_sidebar_collapsed: false,
            _subscriptions: subscriptions,
        }
    }

    fn handle_split_view_event(&mut self, event: &SplitViewEvent, cx: &mut Context<Self>) {
        match event {
            SplitViewEvent::ResizeStart => {}
            SplitViewEvent::SidebarWidthChanged { width } | SplitViewEvent::ResizeEnd { width } => {
                self.split_sidebar_width = width.as_f32();
                cx.notify();
            }
            SplitViewEvent::CollapsedChanged { collapsed } => {
                self.split_sidebar_collapsed = *collapsed;
                cx.notify();
            }
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

    fn handle_switch_event(&mut self, event: &SwitchEvent, cx: &mut Context<Self>) {
        match event {
            SwitchEvent::Change { on } => {
                self.switch_on = *on;
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

    fn handle_radio_group_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<Self>) {
        match event {
            RadioGroupEvent::Change { label, .. } => {
                self.radio_choice = label.to_string();
                cx.notify();
            }
        }
    }

    fn handle_slider_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        match event {
            SliderEvent::Change { value } => {
                self.slider_value = *value;
                self.progress.update(cx, |progress, cx| {
                    progress.set_value(*value, cx);
                });
                cx.notify();
            }
        }
    }

    fn handle_scrollbar_event(
        &mut self,
        presentation: ScrollbarPresentation,
        event: &ScrollbarEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ScrollbarEvent::Change { value } => {
                match presentation {
                    ScrollbarPresentation::Horizontal => {
                        self.horizontal_scroll_value = *value;
                    }
                    ScrollbarPresentation::Vertical => {
                        self.vertical_scroll_value = *value;
                    }
                }
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

    fn render_sidebar(&self) -> AnyElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .bg(rgb(0xffffff))
            .text_color(rgb(0x334155))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pb_2()
                    .child(div().text_size(px(14.0)).line_height(px(18.0)).text_color(rgb(0x0f172a)).child("GPUI-Luma"))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .text_color(rgb(0x64748b))
                            .child("Control gallery"),
                    ),
            )
            .child(gallery_nav_section("Command", &["Button", "Icon Button", "Toggle Button"], true))
            .child(gallery_nav_section("Choice", &["Switch", "Checkbox", "Radio Group"], false))
            .child(gallery_nav_section("Input", &["Slider", "Scrollbar"], false))
            .child(gallery_nav_section("Menu", &["Dropdown Menu", "Context Menu"], false))
            .child(gallery_nav_section("Feedback", &["Progress"], false))
            .into_any_element()
    }
}

impl Render for GalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_scope = self.focus_scope.clone();
        let mut demo_content = div()
            .absolute()
            .left(px(-self.horizontal_scroll_value))
            .top(px(-self.vertical_scroll_value))
            .flex()
            .flex_col()
            .gap_2()
            .p_3();

        for row in 0..12 {
            demo_content = demo_content.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(92.0)).text_color(rgb(0x0f172a)).child(format!("Row {:02}", row + 1)))
                    .child(div().w(px(110.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbae6fd)))
                    .child(div().w(px(150.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbbf7d0)))
                    .child(div().w(px(96.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xfed7aa))),
            );
        }

        let sidebar = self.render_sidebar();
        let work_content = div()
            .size_full()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .occlude()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(self.button.clone())
                            .child(self.icon_button.clone())
                            .child(self.toggle_button.clone())
                            .child(self.switch.clone()),
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
                    .child(div().flex().items_center().gap_3().child(self.radio_group.clone()))
                    .child(div().flex().items_center().gap_4().child(self.slider.clone()).child(self.progress.clone()))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .relative()
                                            .w(px(260.0))
                                            .h(px(180.0))
                                            .overflow_hidden()
                                            .rounded(px(6.0))
                                            .border_1()
                                            .border_color(rgb(0xcbd5e1))
                                            .bg(rgb(0xffffff))
                                            .child(demo_content),
                                    )
                                    .child(self.horizontal_scrollbar.clone()),
                            )
                            .child(self.vertical_scrollbar.clone()),
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
                            .child(format!("Split sidebar: {:.0}px", self.split_sidebar_width))
                            .child(format!("Split collapsed: {}", self.split_sidebar_collapsed))
                            .child(format!("Toggle selected: {}", self.toggle_selected))
                            .child(format!("Switch on: {}", self.switch_on))
                            .child(format!(
                                "Checkboxes: as-is={}, border={}, border + fill={}",
                                self.default_checkbox_checked,
                                self.border_checkbox_checked,
                                self.filled_checkbox_checked
                            ))
                            .child(format!("Radio choice: {}", self.radio_choice))
                            .child(format!("Slider value: {:.0}", self.slider_value))
                            .child(format!(
                                "Scrollbar offset: x={:.0}, y={:.0}",
                                self.horizontal_scroll_value, self.vertical_scroll_value
                            ))
                            .child(format!("Dropdown selected: {}", self.dropdown_selection))
                            .child(format!("Default context selected: {}", self.default_context_selection))
                            .child(format!("Radial context selected: {}", self.radial_context_selection)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(self.disabled_button.clone())
                            .child(self.disabled_icon_button.clone())
                            .child(self.disabled_toggle_button.clone())
                            .child(self.disabled_switch.clone())
                            .child(self.disabled_checkbox.clone()),
                    ),
            )
            .into_any_element();

        self.split_view.update(cx, |split_view, _cx| {
            split_view.set_panes_once(sidebar, work_content);
        });

        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .relative()
            .child(div().absolute().size_full().bg(rgb(0xf8fafc)).on_mouse_down(
                MouseButton::Left,
                move |_event, window, cx| {
                    window.focus(&focus_scope, cx);
                    cx.stop_propagation();
                },
            ))
            .child(self.split_view.clone())
    }
}

fn checkbox_template(theme: impl CheckboxTheme + 'static) -> Arc<dyn CheckboxTemplate> {
    Arc::new(ThemedCheckboxTemplate::new(Arc::new(theme)))
}

fn gallery_nav_section(title: &'static str, items: &'static [&'static str], first_item_active: bool) -> AnyElement {
    let mut section = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_size(px(11.0)).line_height(px(14.0)).text_color(rgb(0x64748b)).child(title));

    for (index, item) in items.iter().enumerate() {
        let active = first_item_active && index == 0;
        section = section.child(
            div()
                .rounded(px(5.0))
                .px_2()
                .py_1()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .text_color(if active { rgb(0x0f172a) } else { rgb(0x475569) })
                .bg(if active { rgb(0xe0f2fe) } else { rgb(0xffffff) })
                .child(*item),
        );
    }

    section.into_any_element()
}

fn default_context_menu_items() -> [DropdownMenuItem; 4] {
    [
        DropdownMenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        DropdownMenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        DropdownMenuItem::new("inspect").label("Inspect"),
        DropdownMenuItem::new("more").label("More").icon(LucideIcon::Ellipsis).submenu([
            DropdownMenuItem::new("download").label("Download").icon(LucideIcon::Download),
            DropdownMenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
        ]),
    ]
}
