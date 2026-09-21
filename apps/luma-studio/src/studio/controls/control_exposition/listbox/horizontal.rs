//! Independent horizontal ListBox example and its item component.

use std::sync::Arc;

use gpui::{
    App, Context, Div, Entity, IntoElement, Render, RenderOnce, SharedString, Stateful, Window, div, prelude::*, px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxInput, ListBoxScrollHandle, ListBoxSnapshot, ListBoxState, ListBoxVisibleItem,
    SelectionMode, SelectionPolicy,
};
use luma::hstack;
use luma_look_shadcn::ShadcnLook;

use super::{ListBoxSampleLayout, ITEM_CONTENT_GAP};
use super::presentation::{ExamplePresentation, SelectionMark};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-horizontal";
pub(in super::super) const CARD_WIDTH: f32 = 136.0;
pub(in super::super) const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: 36.0,
    spacing: 8.0,
    item_padding: 12.0,
    viewport_height: 36.0,
    inset_x: 16.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

struct CardItem {
    id: u32,
    label: SharedString,
}

pub(super) struct HorizontalListExample {
    presentation: ExamplePresentation,
    state: ListBoxState<CardItem, u32>,
    binding: ListBoxBinding,
    scroll: ListBoxScrollHandle<u32>,
}

impl HorizontalListExample {
    pub(super) fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let items = (1..=20).map(|id| CardItem { id, label: format!("Card {id}").into() });
        let snapshot = ListBoxSnapshot::try_with_enabled(items, |item| item.id, |item| item.id != 7)
            .expect("sample items have unique numeric keys");
        Self {
            presentation: ExamplePresentation::new(look, "Horizontal", event_stream),
            state: ListBoxState::from_snapshot(snapshot, SelectionMode::Extended),
            binding: ListBoxBinding::new(cx),
            scroll: ListBoxScrollHandle::default().require_focus_for_scroll(true),
        }
    }

    fn handle_input(&mut self, input: ListBoxInput<u32>, _window: &mut Window, cx: &mut Context<Self>) {
        let update = self.state.apply(input);
        self.scroll.handle_update(&update, cx);
        self.presentation.record(&update.events, cx);
    }

    fn render_item(
        &self,
        item: ListBoxVisibleItem<'_, CardItem, u32>,
        focus_visible: bool,
        cx: &mut Context<Self>,
    ) -> SampleCard {
        let surface = self
            .presentation
            .look
            .listbox_row((ID, item.key as u64), item.state, focus_visible)
            .aria_label(item.item.label.clone())
            .when(!item.state.enabled, |row| row.aria_description("Disabled"));
        let surface = self.binding.bind_row(surface, item.key, item.state, cx, Self::handle_input);
        SampleCard::new(surface, item.item, item.state.selected)
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        let update = self.state.set_selection_policy(policy);
        self.scroll.handle_update(&update, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.presentation.sync_look(look, cx);
    }
}

impl Render for HorizontalListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_visible = self.binding.focus_visible(window);
        let rows = self.state.visible_items().map(|item| self.render_item(item, focus_visible, cx)).collect::<Vec<_>>();

        let stack = hstack! {}
            .id("listbox-horizontal-viewport")
            .overflow_x_scroll()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.viewport_height))
            .gap(px(LAYOUT.spacing))
            .aria_label("Horizontal list")
            .aria_description(format!("Selection mode: {:?}", self.state.selection_mode()))
            .children(rows);
        let surface =
            self.scroll.bind(self.presentation.surface(ID, LAYOUT, self.state.is_focused()), stack, &self.state);
        let surface = self.binding.bind_root(
            surface,
            ListBoxAxis::Horizontal,
            self.state.selection_mode(),
            window,
            cx,
            Self::handle_input,
        );
        self.presentation.section(self.state.selected_keys().count(), surface)
    }
}

#[derive(IntoElement)]
struct SampleCard {
    surface: Stateful<Div>,
    label: SharedString,
    selected: bool,
}

impl SampleCard {
    pub fn new(surface: Stateful<Div>, item: &CardItem, selected: bool) -> Self {
        Self { surface, label: item.label.clone(), selected }
    }
}

impl RenderOnce for SampleCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.surface
            .flex()
            .items_center()
            .gap(px(ITEM_CONTENT_GAP))
            .justify_center()
            .flex_shrink_0()
            .w(px(CARD_WIDTH))
            .h(px(LAYOUT.item_height))
            .px(px(LAYOUT.item_padding))
            .rounded(px(LAYOUT.radius))
            .child(SelectionMark { selected: self.selected })
            .child(div().whitespace_nowrap().child(self.label))
    }
}
