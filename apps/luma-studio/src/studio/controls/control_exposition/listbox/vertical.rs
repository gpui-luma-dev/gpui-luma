//! Independent vertical ListBox example and its item component.

use std::sync::Arc;

use gpui::{
    App, Context, Div, Entity, FontWeight, IntoElement, Render, RenderOnce, ScrollHandle, SharedString, Stateful,
    Window, div, point, prelude::*, px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxInput, ListBoxSnapshot, ListBoxState, ListBoxVisibleItem, SelectionMode,
};
use luma::{hstack, vstack};
use luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::{ListBoxSampleLayout, reveal_offset};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-vertical";
pub(in super::super) const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: 36.0,
    spacing: 4.0,
    item_padding: 12.0,
    viewport_height: 196.0,
    inset_x: 16.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

struct RowItem {
    id: u32,
    label: SharedString,
}

pub(super) struct VerticalListExample {
    look: Arc<ShadcnLook>,
    state: ListBoxState<RowItem, u32>,
    binding: ListBoxBinding,
    scroll: ScrollHandle,
    event_stream: Entity<ControlEventStream>,
}

impl VerticalListExample {
    pub(super) fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let items = (1..=20).map(|id| RowItem { id, label: format!("Item {id}").into() });
        let snapshot = ListBoxSnapshot::try_with_enabled(items, |item| item.id, |item| item.id != 7)
            .expect("sample items have unique numeric keys");
        Self {
            look,
            state: ListBoxState::from_snapshot(snapshot, SelectionMode::Multiple),
            binding: ListBoxBinding::new(SelectionMode::Multiple, cx),
            scroll: ScrollHandle::new(),
            event_stream,
        }
    }

    fn handle_input(&mut self, input: ListBoxInput<u32>, _window: &mut Window, cx: &mut Context<Self>) {
        let update = self.state.apply(input);
        if let Some(key) = update.reveal {
            self.reveal(key);
        }
        for event in &update.events {
            let message = format!("Vertical · ListBoxEvent::{event:?}");
            self.event_stream.update(cx, |stream, cx| stream.append_line(&message, cx));
        }
        if update.changed || update.reveal.is_some() {
            cx.notify();
        }
    }

    fn reveal(&self, key: u32) {
        let Some(index) = self.state.visible_items().find(|item| item.key == key).map(|item| item.visible_index) else {
            return;
        };
        let offset = self.scroll.offset();
        let (extent, viewport, current) = (LAYOUT.item_height, LAYOUT.viewport_height, -offset.y.as_f32());
        let start = index as f32 * (extent + LAYOUT.spacing);
        let target = reveal_offset(current, viewport, start, extent);
        self.scroll.set_offset(point(offset.x, px(-target)));
    }

    fn render_item(
        &self,
        item: ListBoxVisibleItem<'_, RowItem, u32>,
        focus_visible: bool,
        cx: &mut Context<Self>,
    ) -> SampleRow {
        let surface = self
            .look
            .listbox_row((ID, item.key as u64), item.state, focus_visible)
            .aria_label(item.item.label.clone())
            .when(!item.state.enabled, |row| row.aria_description("Disabled"));
        let surface = self.binding.bind_row(surface, item.key, item.state, cx, Self::handle_input);
        SampleRow::new(surface, item.item)
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for VerticalListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_visible = self.binding.focus_handle().is_focused(window) && window.last_input_was_keyboard();
        let rows = self.state.visible_items().map(|item| self.render_item(item, focus_visible, cx)).collect::<Vec<_>>();

        let stack = vstack! {}
            .id("listbox-vertical-viewport")
            .overflow_y_scroll()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.viewport_height))
            .gap(px(LAYOUT.spacing))
            .track_scroll(&self.scroll)
            .aria_label("Vertical list")
            .aria_description("Multiple selection. Click or Space toggles an item.")
            .children(rows);
        let stack = self.binding.bind_root(stack, ListBoxAxis::Vertical, window, cx, Self::handle_input);
        let surface = self.render_surface(stack);
        vstack! { gap=8.0; self.render_header(), surface }.w_full().min_w(px(0.0))
    }
}

impl VerticalListExample {
    fn render_surface(&self, stack: Stateful<Div>) -> Stateful<Div> {
        self.look
            .listbox_surface("listbox-vertical-surface")
            .w_full()
            .min_w(px(0.0))
            .rounded(px(LAYOUT.radius))
            .px(px(LAYOUT.inset_x))
            .py(px(LAYOUT.inset_y))
            // Let the child viewport scroll, then contain the wheel at either endpoint.
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(stack)
    }

    fn render_header(&self) -> impl IntoElement {
        let selected_count = self.state.selected_keys().count();
        let chrome = self.look.chrome();
        let title_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let body_style = self.look.typography_scale(ShadcnTextSize::Xs);
        hstack! { justify=between;
            div().typography_style(title_style).font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text).child("Vertical"),
            div().typography_style(body_style).text_color(chrome.muted_text)
                .child(format!("Selected: {selected_count}")),
        }
    }
}

#[derive(IntoElement)]
struct SampleRow {
    surface: Stateful<Div>,
    label: SharedString,
}

impl SampleRow {
    pub fn new(surface: Stateful<Div>, item: &RowItem) -> Self {
        Self { surface, label: item.label.clone() }
    }
}

impl RenderOnce for SampleRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.surface
            .flex()
            .items_center()
            .flex_shrink_0()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.item_height))
            .px(px(LAYOUT.item_padding))
            .rounded(px(LAYOUT.radius))
            .overflow_hidden()
            .child(div().min_w(px(0.0)).truncate().child(self.label))
    }
}
