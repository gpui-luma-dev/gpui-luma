use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FontWeight, MouseButton, ScrollHandle, Subscription, Window, div, point, prelude::*,
    px,
};
use gpui_luma::controls::command::button::{Button, ButtonEvent};
use gpui_luma::controls::overlay_window::OverlayWindow;
use gpui_luma::controls::textfield::TextField;
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use super::catalog::{ControlCategory, ControlDocEntry, entries_for_category};
use super::doc_card::{render_control_doc_card, render_category_heading};
use super::eventing::{ButtonEventStream, ButtonVariant, render_button_possible_events_section};
use crate::studio::doc_shell::{
    StickySectionHeadingTracker, render_sticky_section_heading_lane, with_sticky_heading_tracker,
};

const CONTROL_INDEX_WIDTH: f32 = 168.0;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    scroll_handle: ScrollHandle,
    sticky_heading_tracker: std::rc::Rc<std::cell::RefCell<StickySectionHeadingTracker>>,
    last_scroll_offset: std::rc::Rc<std::cell::Cell<f32>>,
    last_max_scroll: std::rc::Rc<std::cell::Cell<f32>>,
    button_primary: Entity<Button>,
    button_secondary: Entity<Button>,
    button_outline: Entity<Button>,
    button_ghost: Entity<Button>,
    button_event_stream: Entity<ButtonEventStream>,
    textfield_preview: TextField,
    dialog_trigger: Entity<Button>,
    dialog_cancel: Entity<Button>,
    dialog_confirm: Entity<Button>,
    dialog_overlay: OverlayWindow,
    _subscriptions: Vec<Subscription>,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let button_primary = look.primary_button("controls-doc-button-primary").label("Primary").spawn(cx);
        let button_secondary = look.secondary_button("controls-doc-button-secondary").label("Secondary").spawn(cx);
        let button_outline = look.outline_button("controls-doc-button-outline").label("Outline").spawn(cx);
        let button_ghost = look.ghost_button("controls-doc-button-ghost").label("Ghost").spawn(cx);
        let button_event_stream = cx.new(|cx| ButtonEventStream::new(cx, look.clone()));
        let textfield_preview = look
            .textfield("controls-doc-textfield-preview")
            .placeholder("Email address")
            .full_width(true)
            .spawn(cx);

        let dialog_cancel = look.outline_button("controls-doc-dialog-cancel").label("Cancel").spawn(cx);
        let dialog_confirm = look.primary_button("controls-doc-dialog-confirm").label("Confirm").spawn(cx);
        let dialog_overlay = look
            .overlay_window("controls-doc-dialog-overlay")
            .mode(gpui_luma::controls::overlay_window::OverlayWindowMode::Modal)
            .content({
                let cancel = dialog_cancel.clone();
                let confirm = dialog_confirm.clone();
                move |_, _, _| {
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .p(px(20.0))
                        .child("Archive this project? This action cannot be undone.")
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .child(cancel.clone())
                                .child(confirm.clone()),
                        )
                        .into_any_element()
                }
            })
            .theme_children([dialog_cancel.clone(), dialog_confirm.clone()])
            .spawn(cx);
        let dialog_trigger = look.primary_button("controls-doc-dialog-trigger").label("Open modal").spawn(cx);

        let mut subscriptions = Vec::new();

        subscriptions.extend(subscribe_button_event(
            &button_primary,
            ButtonVariant::Primary,
            "controls-doc-button-primary",
            button_event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button_event(
            &button_secondary,
            ButtonVariant::Secondary,
            "controls-doc-button-secondary",
            button_event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button_event(
            &button_outline,
            ButtonVariant::Outline,
            "controls-doc-button-outline",
            button_event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_button_event(
            &button_ghost,
            ButtonVariant::Ghost,
            "controls-doc-button-ghost",
            button_event_stream.clone(),
            cx,
        ));

        subscriptions.push(cx.subscribe(&dialog_trigger, {
            let overlay = dialog_overlay.clone();
            move |_, _, _: &ButtonEvent, cx| {
                overlay.update(cx, |overlay, cx| overlay.open(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&dialog_cancel, {
            let overlay = dialog_overlay.clone();
            move |_, _, _: &ButtonEvent, cx| {
                overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));
        subscriptions.push(cx.subscribe(&dialog_confirm, {
            let overlay = dialog_overlay.clone();
            move |_, _, _: &ButtonEvent, cx| {
                overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));

        Self {
            look,
            scroll_handle: ScrollHandle::new(),
            sticky_heading_tracker: std::rc::Rc::new(std::cell::RefCell::new(StickySectionHeadingTracker::default())),
            last_scroll_offset: std::rc::Rc::new(std::cell::Cell::new(0.0)),
            last_max_scroll: std::rc::Rc::new(std::cell::Cell::new(0.0)),
            button_primary,
            button_secondary,
            button_outline,
            button_ghost,
            button_event_stream,
            textfield_preview,
            dialog_trigger,
            dialog_cancel,
            dialog_confirm,
            dialog_overlay,
            _subscriptions: subscriptions,
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for entity in [&self.button_primary, &self.button_secondary, &self.button_outline, &self.button_ghost] {
            entity.update(cx, |_, cx| cx.notify());
        }
        self.button_event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.textfield_preview.update(cx, |_, cx| cx.notify());
        for entity in [&self.dialog_trigger, &self.dialog_cancel, &self.dialog_confirm] {
            entity.update(cx, |_, cx| cx.notify());
        }
        self.dialog_overlay.update(cx, |_, cx| cx.notify());
        self.sticky_heading_tracker.borrow_mut().reset();
        cx.notify();
    }

    fn set_vertical_offset(&self, value: f32, cx: &mut Context<Self>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    fn scroll_to_entry(&mut self, entry: ControlDocEntry, cx: &mut Context<Self>) {
        let Some(top) = self.sticky_heading_tracker.borrow().anchor_top(entry.title) else {
            return;
        };
        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        self.set_vertical_offset(top.clamp(0.0, max), cx);
    }

    fn render_preview(&self, entry: ControlDocEntry) -> AnyElement {
        match entry.id {
            "button" => div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div().w_full().flex().justify_center().child(
                        div()
                            .flex()
                            .flex_wrap()
                            .justify_center()
                            .gap(px(12.0))
                            .child(self.button_primary.clone())
                            .child(self.button_secondary.clone())
                            .child(self.button_outline.clone())
                            .child(self.button_ghost.clone()),
                    ),
                )
                .child(self.button_event_stream.clone())
                .into_any_element(),
            "textfield" => div().w_full().max_w(px(360.0)).child(self.textfield_preview.clone()).into_any_element(),
            "modal-overlay" => div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(self.dialog_trigger.clone())
                .child(self.dialog_overlay.clone())
                .into_any_element(),
            _ => div().into_any_element(),
        }
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let scroll_handle = self.scroll_handle.clone();
            let last_scroll_offset = self.last_scroll_offset.clone();
            let last_max_scroll = self.last_max_scroll.clone();
            let sticky_heading_tracker = self.sticky_heading_tracker.clone();
            let scroll_y = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
            let sticky_snapshot = sticky_heading_tracker.borrow().snapshot(scroll_y);
            let active_section_title = sticky_heading_tracker.borrow().active_title(scroll_y);

            with_sticky_heading_tracker(sticky_heading_tracker.clone(), || {
                div()
                    .id("luma-studio-controls")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .bg(chrome.content_background)
                    .px(px(28.0))
                    .pt(px(12.0))
                    .pb(px(28.0))
                    .child(
                        div()
                            .w_full()
                            .min_h(px(0.0))
                            .flex_1()
                            .flex()
                            .items_stretch()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .child(
                                        div()
                                            .id("controls-content")
                                            .size_full()
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(0.0))
                                            .track_scroll(&self.scroll_handle)
                                            .on_prepaint({
                                                let scroll_handle = scroll_handle.clone();
                                                let last_scroll_offset = last_scroll_offset.clone();
                                                let last_max_scroll = last_max_scroll.clone();
                                                move |_, window: &mut gpui::Window, cx: &mut gpui::App| {
                                                    let offset = (-scroll_handle.offset().y.as_f32()).max(0.0);
                                                    let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                                                    let offset_changed =
                                                        (last_scroll_offset.get() - offset).abs() > 0.5;
                                                    let max_changed = (last_max_scroll.get() - max_scroll).abs() > 0.5;

                                                    if offset_changed || max_changed {
                                                        last_scroll_offset.set(offset);
                                                        last_max_scroll.set(max_scroll);
                                                        cx.notify(window.current_view());
                                                    }
                                                }
                                            })
                                            .child(
                                                div().w_full().flex().justify_start().pb(px(12.0)).child(
                                                    div()
                                                        .w_full()
                                                        .max_w(px(980.0))
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(20.0))
                                                        .on_prepaint({
                                                            let sticky_heading_tracker = sticky_heading_tracker.clone();
                                                            move |bounds, window, cx| {
                                                                let changed = sticky_heading_tracker
                                                                    .borrow_mut()
                                                                    .set_content_origin_y(bounds.origin.y.as_f32());
                                                                if changed {
                                                                    cx.notify(window.current_view());
                                                                }
                                                            }
                                                        })
                                                        .children(ControlCategory::ALL.into_iter().filter_map(
                                                            |category| {
                                                                let entries: Vec<_> =
                                                                    entries_for_category(category).collect();
                                                                if entries.is_empty() {
                                                                    return None;
                                                                }
                                                                Some(render_category_section(
                                                                    &self.look,
                                                                    category,
                                                                    &entries,
                                                                    |entry| self.render_preview(entry),
                                                                ))
                                                            },
                                                        )),
                                                ),
                                            ),
                                    )
                                    .child(render_sticky_section_heading_lane(
                                        sticky_snapshot,
                                        chrome.content_background,
                                        980.0,
                                    )),
                            )
                            .child(self.render_control_index(active_section_title, cx)),
                    )
            })
        })
    }
}

fn render_category_section(
    look: &ShadcnLook,
    category: ControlCategory,
    entries: &[&ControlDocEntry],
    preview_for: impl Fn(ControlDocEntry) -> AnyElement + Copy,
) -> AnyElement {
    let chrome = look.chrome();
    let category_order = category.index_order() * 1000;

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(render_category_heading(
            category.label(),
            category_description(category),
            category_order,
            chrome.title_text,
            chrome.muted_text,
            chrome.border,
        ))
        .children(entries.iter().map(|entry| {
            let between_preview_and_snippet = if entry.id == "button" {
                Some(render_button_possible_events_section(look))
            } else {
                None
            };
            render_control_doc_card(
                look,
                **entry,
                preview_for(**entry),
                between_preview_and_snippet,
                entry.id == "button",
            )
        }))
        .into_any_element()
}

fn subscribe_button_event(
    button: &Entity<Button>,
    variant: ButtonVariant,
    button_id: &'static str,
    event_stream: Entity<ButtonEventStream>,
    cx: &mut Context<ControlsPanel>,
) -> Vec<Subscription> {
    vec![cx.subscribe(button, move |_, _, event: &ButtonEvent, cx| {
        event_stream.update(cx, |stream, cx| {
            stream.record_event(variant, button_id, event, cx);
            cx.notify();
        });
    })]
}

fn category_description(category: ControlCategory) -> &'static str {
    match category {
        ControlCategory::Command => "Action triggers such as buttons and toggles.",
        ControlCategory::Choice => "Single and multi-select choice controls.",
        ControlCategory::Inputs => "Text entry surfaces for forms and filters.",
        ControlCategory::Selection => "List-backed pickers, comboboxes, and selectors.",
        ControlCategory::NavigationPanels => "Tabs, sidebars, accordions, and pagers.",
        ControlCategory::OverlaysDialogs => "Menus, popups, and modal overlay windows.",
    }
}

impl ControlsPanel {
    fn render_control_index(&self, active_title: Option<&'static str>, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();

        div()
            .id("controls-control-index")
            .w(px(CONTROL_INDEX_WIDTH))
            .flex_shrink_0()
            .child(
                div().w_full().h_full().flex().items_center().justify_center().child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .line_height(px(24.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(chrome.title_text)
                                .child("Control Index"),
                        )
                        .children(ControlCategory::ALL.into_iter().filter_map(|category| {
                            let entries: Vec<_> = entries_for_category(category).collect();
                            if entries.is_empty() {
                                return None;
                            }
                            Some(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .px(px(8.0))
                                            .text_size(px(11.0))
                                            .line_height(px(14.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(chrome.muted_text)
                                            .child(category.label()),
                                    )
                                    .children(entries.into_iter().map(|entry| {
                                        self.render_control_index_item(*entry, active_title == Some(entry.title), cx)
                                    })),
                            )
                        })),
                ),
            )
            .into_any_element()
    }

    fn render_control_index_item(&self, entry: ControlDocEntry, active: bool, cx: &mut Context<Self>) -> AnyElement {
        let chrome = self.look.chrome();
        let color = if active { chrome.title_text } else { chrome.muted_text };

        div()
            .id(format!("controls-index-item-{}", entry.id))
            .w_full()
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(6.0))
            .text_size(px(13.0))
            .line_height(px(17.0))
            .font_weight(if active {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(gpui::hsla(color.h, color.s, color.l, if active { 1.0 } else { 0.82 }))
            .when(active, |item| {
                item.bg(gpui::hsla(chrome.muted_text.h, chrome.muted_text.s, chrome.muted_text.l, 0.10))
            })
            .cursor_pointer()
            .hover(move |style| {
                style
                    .bg(gpui::hsla(chrome.muted_text.h, chrome.muted_text.s, chrome.muted_text.l, 0.12))
                    .text_color(chrome.title_text)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    this.scroll_to_entry(entry, cx);
                }),
            )
            .child(entry.title)
            .into_any_element()
    }
}
