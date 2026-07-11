use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::prelude::*;

use super::{RadioControlGroup, RadioControlGroupChild, RadioControlGroupEvent};

pub(in crate::gallery) struct RadioControlGroupDemo {
    look: Arc<ShadcnLook>,
    focus_target: Entity<Button>,
    control: Entity<RadioControlGroup>,
    selected_label: String,
    subscriptions: Vec<Subscription>,
}

impl RadioControlGroupDemo {
    pub(in crate::gallery) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let focus_target =
            look.secondary_button("proto-radio-control-group-focus-target").label("Focus Target").spawn(cx);
        let control = RadioControlGroup::builder("proto-radio-control-group-demo")
            .vertical()
            .child(RadioControlGroupChild::new("compact").label("Compact").enabled(true))
            .child(RadioControlGroupChild::new("comfortable").label("Comfortable").enabled(true))
            .child(RadioControlGroupChild::new("expanded").label("Expanded").enabled(true))
            .selected("comfortable")
            .spawn(cx, look.clone());
        let selected_label = control
            .read(cx)
            .selected_label()
            .map(ToString::to_string)
            .unwrap_or_else(|| "comfortable".to_string());
        let subscriptions = vec![cx.subscribe(&control, |this, _, event: &RadioControlGroupEvent, cx| {
            let RadioControlGroupEvent::SelectionChanged { selected_id: _ } = event;
            this.selected_label = this.control.read(cx).selected_label().map(ToString::to_string).unwrap_or_default();
            cx.notify();
        })];

        Self { look, focus_target, control, selected_label, subscriptions }
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<Self>) {
        let _ = &self.subscriptions;
        self.focus_target.update(cx, |_, cx| cx.notify());
        self.control.update(cx, |control, cx| control.notify_controls(cx));
    }
}

impl Render for RadioControlGroupDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();

        div().flex().flex_col().items_start().gap(px(16.0)).child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(self.focus_target.clone())
                .child(self.control.clone())
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .text_color(chrome.body_text)
                        .child(format!("Choice: {}", self.selected_label)),
                ),
        )
    }
}
