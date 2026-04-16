use gpui::{Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, rgb};
use gpui_luma::controls::button::{Button, ButtonEvent, ButtonKind};

pub struct GalleryApp {
    button: Entity<Button>,
    disabled_button: Entity<Button>,
    clicks: usize,
    _subscriptions: Vec<Subscription>,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let button = Button::new("button-example")
            .label("Click me")
            .kind(ButtonKind::Primary)
            .spawn(cx);
        let disabled_button = Button::new("disabled-button")
            .label("Disabled")
            .enabled(false)
            .spawn(cx);

        let subscriptions = vec![cx.subscribe(&button, |this, _, event: &ButtonEvent, cx| {
            this.handle_button_event(event, cx);
        })];

        Self {
            button,
            disabled_button,
            clicks: 0,
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
            .child(self.button.clone())
            .child(self.disabled_button.clone())
    }
}
