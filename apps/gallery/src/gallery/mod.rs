use gpui::{Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, rgb};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};
use gpui_luma::controls::icon_button::{IconButton, IconButtonEvent, IconButtonKind};
use gpui_luma::controls::toggle_button::{ToggleButton, ToggleButtonEvent};
use lucide_icons::Icon as LucideIcon;

pub struct GalleryApp {
    button: Entity<Button>,
    icon_button: Entity<IconButton>,
    toggle_button: Entity<ToggleButton>,
    disabled_button: Entity<Button>,
    disabled_icon_button: Entity<IconButton>,
    disabled_toggle_button: Entity<ToggleButton>,
    clicks: usize,
    icon_clicks: usize,
    toggle_selected: bool,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
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
        ];

        Self {
            button,
            icon_button,
            toggle_button,
            disabled_button,
            disabled_icon_button,
            disabled_toggle_button,
            clicks: 0,
            icon_clicks: 0,
            toggle_selected: true,
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
                    .text_color(rgb(0x334155))
                    .child(format!("Toggle selected: {}", self.toggle_selected)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.disabled_button.clone())
                    .child(self.disabled_icon_button.clone())
                    .child(self.disabled_toggle_button.clone()),
            )
    }
}
