//! Independent vertical ListBox example and its item component.

use std::sync::Arc;

use gpui::{
    App, Context, Div, Entity, IntoElement, Render, RenderOnce, SharedString, Stateful, Window, div, prelude::*, px,
};
use luma::controls::listbox::{
    ListBoxAxis, ListBoxBinding, ListBoxInput, ListBoxScrollHandle, ListBoxSnapshot, ListBoxState, ListBoxVisibleItem,
    SelectionMode, SelectionPolicy,
};
use luma::vstack;
use luma_look_shadcn::ShadcnLook;

use super::{ListBoxSampleLayout, ITEM_CONTENT_GAP, VERTICAL_LIST_WIDTH};
use super::presentation::{ExamplePresentation, SelectionMark};
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
    presentation: ExamplePresentation,
    state: ListBoxState<RowItem, u32>,
    binding: ListBoxBinding,
    scroll: ListBoxScrollHandle<u32>,
}

impl VerticalListExample {
    pub(super) fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let items = (1..=20).map(|id| RowItem { id, label: format!("Item {id}").into() });
        let snapshot = ListBoxSnapshot::try_with_enabled(items, |item| item.id, |item| item.id != 7)
            .expect("sample items have unique numeric keys");
        Self {
            presentation: ExamplePresentation::new(look, "Vertical", event_stream),
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
        item: ListBoxVisibleItem<'_, RowItem, u32>,
        focus_visible: bool,
        cx: &mut Context<Self>,
    ) -> SampleRow {
        let surface = self
            .presentation
            .look
            .listbox_row((ID, item.key as u64), item.state, focus_visible)
            .aria_label(item.item.label.clone())
            .when(!item.state.enabled, |row| row.aria_description("Disabled"));
        let surface = self.binding.bind_row(surface, item.key, item.state, cx, Self::handle_input);
        SampleRow::new(surface, item.item, item.state.selected)
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

impl Render for VerticalListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_visible = self.binding.focus_visible(window);
        let rows = self.state.visible_items().map(|item| self.render_item(item, focus_visible, cx)).collect::<Vec<_>>();

        let stack = vstack! {}
            .id("listbox-vertical-viewport")
            .overflow_y_scroll()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.viewport_height))
            .gap(px(LAYOUT.spacing))
            .aria_label("Vertical list")
            .aria_description(format!("Selection mode: {:?}", self.state.selection_mode()))
            .children(rows);
        let surface =
            self.scroll.bind(self.presentation.surface(ID, LAYOUT, self.state.is_focused()), stack, &self.state);
        let surface = self.binding.bind_root(
            surface,
            ListBoxAxis::Vertical,
            self.state.selection_mode(),
            window,
            cx,
            Self::handle_input,
        );
        self.presentation
            .section(self.state.selected_keys().count(), surface)
            .w(px(VERTICAL_LIST_WIDTH))
            .flex_shrink_0()
    }
}

#[derive(IntoElement)]
struct SampleRow {
    surface: Stateful<Div>,
    label: SharedString,
    selected: bool,
}

impl SampleRow {
    pub fn new(surface: Stateful<Div>, item: &RowItem, selected: bool) -> Self {
        Self { surface, label: item.label.clone(), selected }
    }
}

impl RenderOnce for SampleRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.surface
            .flex()
            .items_center()
            .gap(px(ITEM_CONTENT_GAP))
            .flex_shrink_0()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.item_height))
            .px(px(LAYOUT.item_padding))
            .rounded(px(LAYOUT.radius))
            .overflow_hidden()
            .child(SelectionMark { selected: self.selected })
            .child(div().min_w(px(0.0)).truncate().child(self.label))
    }
}
