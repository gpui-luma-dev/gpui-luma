use gpui::{AnyElement, Context, Entity, IntoElement, Subscription, div, prelude::*, px, rgb};
use gpui_luma::controls::scrollbar::{Scrollbar, ScrollbarEvent};

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ScrollbarPane {
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    disabled_horizontal_scrollbar: Entity<Scrollbar>,
    disabled_vertical_scrollbar: Entity<Scrollbar>,
    horizontal_value: f32,
    vertical_value: f32,
}

#[derive(Clone, Copy)]
enum ScrollbarPresentation {
    Horizontal,
    Vertical,
}

impl ScrollbarPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            horizontal_scrollbar: Scrollbar::new("scrollbar-horizontal-example")
                .horizontal()
                .range(0..220)
                .step(20)
                .page_step(80)
                .value(40)
                .thumb_fraction(0.54)
                .template(theme.scrollbar_template())
                .spawn(cx),
            vertical_scrollbar: Scrollbar::new("scrollbar-vertical-example")
                .vertical()
                .range(0..240)
                .step(20)
                .page_step(80)
                .value(80)
                .thumb_fraction(0.45)
                .template(theme.scrollbar_template())
                .spawn(cx),
            disabled_horizontal_scrollbar: Scrollbar::new("disabled-scrollbar-horizontal-example")
                .horizontal()
                .range(0..220)
                .step(20)
                .page_step(80)
                .value(40)
                .thumb_fraction(0.54)
                .enabled(false)
                .template(theme.scrollbar_template())
                .spawn(cx),
            disabled_vertical_scrollbar: Scrollbar::new("disabled-scrollbar-vertical-example")
                .vertical()
                .range(0..240)
                .step(20)
                .page_step(80)
                .value(80)
                .thumb_fraction(0.45)
                .enabled(false)
                .template(theme.scrollbar_template())
                .spawn(cx),
            horizontal_value: 40.0,
            vertical_value: 80.0,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.horizontal_scrollbar, |app, _, event: &ScrollbarEvent, cx| {
            app.panes.scrollbar.handle_event(ScrollbarPresentation::Horizontal, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.vertical_scrollbar, |app, _, event: &ScrollbarEvent, cx| {
            app.panes.scrollbar.handle_event(ScrollbarPresentation::Vertical, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane(
            "Scrollbar",
            div()
                .flex()
                .items_start()
                .gap_4()
                .child(self.scrollbar_example(theme))
                .child(self.disabled_scrollbar_example(theme))
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.horizontal_scrollbar, cx);
        notify_entity(&self.vertical_scrollbar, cx);
        notify_entity(&self.disabled_horizontal_scrollbar, cx);
        notify_entity(&self.disabled_vertical_scrollbar, cx);
    }

    fn handle_event(
        &mut self,
        presentation: ScrollbarPresentation,
        event: &ScrollbarEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            ScrollbarEvent::Change { value } => {
                match presentation {
                    ScrollbarPresentation::Horizontal => {
                        self.horizontal_value = *value;
                    }
                    ScrollbarPresentation::Vertical => {
                        self.vertical_value = *value;
                    }
                }
                cx.notify();
            }
        }
    }

    fn scrollbar_example(&self, theme: &GalleryThemePack) -> AnyElement {
        scrollbar_pair(
            self.horizontal_value,
            self.vertical_value,
            self.horizontal_scrollbar.clone(),
            self.vertical_scrollbar.clone(),
            theme,
        )
    }

    fn disabled_scrollbar_example(&self, theme: &GalleryThemePack) -> AnyElement {
        scrollbar_pair(
            self.horizontal_value,
            self.vertical_value,
            self.disabled_horizontal_scrollbar.clone(),
            self.disabled_vertical_scrollbar.clone(),
            theme,
        )
    }
}

fn scrollbar_pair(
    horizontal_value: f32,
    vertical_value: f32,
    horizontal_scrollbar: Entity<Scrollbar>,
    vertical_scrollbar: Entity<Scrollbar>,
    theme: &GalleryThemePack,
) -> AnyElement {
    let chrome = theme.chrome();
    let mut demo_content = div()
        .absolute()
        .left(px(-horizontal_value))
        .top(px(-vertical_value))
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
                .child(div().w(px(92.0)).text_color(chrome.title_text).child(format!("Row {:02}", row + 1)))
                .child(div().w(px(110.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbae6fd)))
                .child(div().w(px(150.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xbbf7d0)))
                .child(div().w(px(96.0)).h(px(22.0)).rounded(px(4.0)).bg(rgb(0xfed7aa))),
        );
    }

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
                        .border_color(chrome.border)
                        .bg(chrome.panel_background)
                        .child(demo_content),
                )
                .child(horizontal_scrollbar),
        )
        .child(vertical_scrollbar)
        .into_any_element()
}
