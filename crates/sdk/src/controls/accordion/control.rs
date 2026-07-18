use std::collections::HashSet;
use std::sync::Arc;

use gpui::{App, Context, EventEmitter, Focusable, FocusHandle, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::{
    AccordionBuilder, AccordionItem, AccordionItemRenderModel, AccordionModel, AccordionRenderModel,
    AccordionSelectionMode, AccordionTemplateHandlers,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};
use crate::keyhandling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectNextRow,
    SelectPreviousItem, SelectPreviousRow,
};
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
pub enum AccordionEvent {
    ExpandedChanged { item_id: SharedString, expanded: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccordionDirection {
    Previous,
    Next,
}

pub struct AccordionControl {
    model: AccordionModel,
    focus_handle: FocusHandle,
    expanded_ids: HashSet<SharedString>,
    focused_item_index: Option<usize>,
    hovered_item_index: Option<usize>,
    pressed_item_index: Option<usize>,
}

impl EventEmitter<AccordionEvent> for AccordionControl {}

impl AccordionControl {
    pub(crate) fn from_builder(builder: AccordionBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;
        let mut expanded_ids = HashSet::new();

        for item in &builder.model.items {
            if item.initially_expanded {
                expanded_ids.insert(item.id.clone());
                if builder.model.selection_mode == AccordionSelectionMode::Single {
                    break;
                }
            }
        }

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            expanded_ids,
            focused_item_index: None,
            hovered_item_index: None,
            pressed_item_index: None,
        }
    }

    pub fn is_expanded(&self, item_id: &SharedString) -> bool {
        self.expanded_ids.contains(item_id)
    }

    pub fn set_template(&mut self, template: Arc<dyn super::AccordionTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.focused_item_index = None;
            self.hovered_item_index = None;
            self.pressed_item_index = None;
        }
        cx.notify();
    }

    pub fn toggle_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.model.enabled || index >= self.model.items.len() {
            return;
        }

        let item = &self.model.items[index];
        if !item.enabled {
            return;
        }

        let id = item.id.clone();
        for event in apply_toggle(&mut self.expanded_ids, self.model.selection_mode, self.model.collapsible, id) {
            cx.emit(event);
        }

        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> AccordionRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);

        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let item_enabled = self.model.enabled && item.enabled;
                let expanded = self.expanded_ids.contains(&item.id);
                let has_keyboard_focus = focus.focused && self.focused_item_index == Some(index);

                AccordionItemRenderModel {
                    id: &item.id,
                    trigger: &item.trigger,
                    content: &item.content,
                    expanded,
                    enabled: item_enabled,
                    state: CompositeItemState {
                        hovered: item_enabled && self.hovered_item_index == Some(index),
                        pressed: item_enabled && self.pressed_item_index == Some(index),
                        disabled: !item_enabled,
                        selected: expanded,
                        active: item_enabled && has_keyboard_focus,
                        focus_visible: item_enabled && focus.focus_visible && has_keyboard_focus,
                    },
                }
            })
            .collect();

        AccordionRenderModel {
            id: &self.model.id,
            items,
            selection_mode: self.model.selection_mode,
            collapsible: self.model.collapsible,
            enabled: self.model.enabled,
            item_dividers: self.model.item_dividers,
            content_padding_y: self.model.content_padding_y,
            content_padding_top: self.model.content_padding_top,
            content_padding_bottom: self.model.content_padding_bottom,
            trigger_min_height: self.model.trigger_min_height,
            trigger_padding_y: self.model.trigger_padding_y,
            focus,
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.items.get(index).is_some_and(|item| item.enabled) {
            return;
        }

        if hovered {
            if self.hovered_item_index != Some(index) {
                self.hovered_item_index = Some(index);
                cx.notify();
            }
        } else if self.hovered_item_index == Some(index) {
            self.hovered_item_index = None;
            if self.pressed_item_index == Some(index) {
                self.pressed_item_index = None;
            }
            cx.notify();
        }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> AccordionTemplateHandlers {
        AccordionTemplateHandlers {
            trigger_hovers: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, hovered, _, cx| {
                        this.handle_item_hover(idx, *hovered, cx);
                    })) as _
                })
                .collect(),
            trigger_mouse_downs: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, window, cx| {
                        if this.model.enabled && this.model.items[idx].enabled {
                            this.pressed_item_index = Some(idx);
                            this.focused_item_index = Some(idx);
                            this.focus_handle.focus(window, cx);
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_mouse_ups: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        if this.pressed_item_index == Some(idx) {
                            this.pressed_item_index = None;
                            cx.notify();
                        }
                    })) as _
                })
                .collect(),
            trigger_clicks: (0..self.model.items.len())
                .map(|idx| {
                    Box::new(cx.listener(move |this, _, _, cx| {
                        this.toggle_item(idx, cx);
                    })) as _
                })
                .collect(),
        }
    }

    fn handle_previous(&mut self, _: &SelectPreviousItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(AccordionDirection::Previous, cx);
    }

    fn handle_next(&mut self, _: &SelectNextItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(AccordionDirection::Next, cx);
    }

    fn handle_previous_row(&mut self, _: &SelectPreviousRow, _: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(AccordionDirection::Previous, cx);
    }

    fn handle_next_row(&mut self, _: &SelectNextRow, _: &mut Window, cx: &mut Context<Self>) {
        self.move_focus(AccordionDirection::Next, cx);
    }

    fn move_focus(&mut self, direction: AccordionDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(next) = next_enabled_index(&self.model.items, self.focused_item_index, direction) {
            self.focused_item_index = Some(next);
            cx.notify();
        }
    }

    fn handle_first(&mut self, _: &SelectFirstItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = first_enabled_index(&self.model.items) {
            self.focused_item_index = Some(idx);
            cx.notify();
        }
    }

    fn handle_last(&mut self, _: &SelectLastItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = last_enabled_index(&self.model.items) {
            self.focused_item_index = Some(idx);
            cx.notify();
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.focused_item_index {
            self.toggle_item(idx, cx);
        }
    }
}

impl Focusable for AccordionControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AccordionControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);

        div()
            .child(
                self.model
                    .template
                    .render(&model, handlers, window, cx)
                    .track_focus(&self.focus_handle)
                    .key_context(ControlKeyProfile::TabList.context())
                    .on_action(cx.listener(Self::handle_previous))
                    .on_action(cx.listener(Self::handle_next))
                    .on_action(cx.listener(Self::handle_previous_row))
                    .on_action(cx.listener(Self::handle_next_row))
                    .on_action(cx.listener(Self::handle_first))
                    .on_action(cx.listener(Self::handle_last))
                    .on_action(cx.listener(Self::handle_activate)),
            )
            .into_any_element()
    }
}

pub(crate) fn first_enabled_index(items: &[AccordionItem]) -> Option<usize> {
    items.iter().position(|item| item.enabled)
}

pub(crate) fn last_enabled_index(items: &[AccordionItem]) -> Option<usize> {
    items.iter().rposition(|item| item.enabled)
}

pub(crate) fn next_enabled_index(
    items: &[AccordionItem],
    current_index: Option<usize>,
    direction: AccordionDirection,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        AccordionDirection::Previous => len - 1,
        AccordionDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(idx) => (idx + step) % len,
        None if direction == AccordionDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].enabled {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

pub(crate) fn apply_toggle(
    expanded_ids: &mut HashSet<SharedString>,
    selection_mode: AccordionSelectionMode,
    collapsible: bool,
    item_id: SharedString,
) -> Vec<AccordionEvent> {
    let was_expanded = expanded_ids.contains(&item_id);
    let mut events = Vec::new();

    match selection_mode {
        AccordionSelectionMode::Single => {
            if was_expanded {
                if collapsible {
                    expanded_ids.remove(&item_id);
                    events.push(AccordionEvent::ExpandedChanged { item_id, expanded: false });
                }
            } else {
                for prev_id in expanded_ids.drain() {
                    events.push(AccordionEvent::ExpandedChanged { item_id: prev_id, expanded: false });
                }
                expanded_ids.insert(item_id.clone());
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: true });
            }
        }
        AccordionSelectionMode::Multiple => {
            if was_expanded {
                expanded_ids.remove(&item_id);
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: false });
            } else {
                expanded_ids.insert(item_id.clone());
                events.push(AccordionEvent::ExpandedChanged { item_id, expanded: true });
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::{
        AccordionDirection, AccordionSelectionMode, apply_toggle, first_enabled_index, last_enabled_index,
        next_enabled_index,
    };
    use crate::controls::accordion::{AccordionContent, AccordionItem, AccordionTrigger};
    use gpui::{SharedString, div, prelude::*};

    fn sample_items() -> Vec<AccordionItem> {
        vec![
            AccordionItem::new(
                "one",
                AccordionTrigger::new("One"),
                AccordionContent::custom(|_, _| div().child("Content one").into_any_element()),
            ),
            AccordionItem::new(
                "two",
                AccordionTrigger::new("Two"),
                AccordionContent::custom(|_, _| div().child("Content two").into_any_element()),
            )
            .enabled(false),
            AccordionItem::new(
                "three",
                AccordionTrigger::new("Three"),
                AccordionContent::custom(|_, _| div().child("Content three").into_any_element()),
            ),
        ]
    }

    #[test]
    fn navigation_skips_disabled_items_and_wraps() {
        let items = sample_items();

        assert_eq!(next_enabled_index(&items, Some(0), AccordionDirection::Next), Some(2));
        assert_eq!(next_enabled_index(&items, Some(2), AccordionDirection::Next), Some(0));
        assert_eq!(next_enabled_index(&items, Some(0), AccordionDirection::Previous), Some(2));
    }

    #[test]
    fn first_and_last_enabled_index_skip_disabled_items() {
        let items = sample_items();

        assert_eq!(first_enabled_index(&items), Some(0));
        assert_eq!(last_enabled_index(&items), Some(2));
    }

    #[test]
    fn single_mode_collapses_other_items_when_expanding() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Single, true, SharedString::from("three"));

        assert_eq!(expanded.len(), 1);
        assert!(expanded.contains(&SharedString::from("three")));
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|event| matches!(event, super::AccordionEvent::ExpandedChanged { item_id, expanded: false } if item_id.as_ref() == "one")));
        assert!(events.iter().any(|event| matches!(event, super::AccordionEvent::ExpandedChanged { item_id, expanded: true } if item_id.as_ref() == "three")));
    }

    #[test]
    fn single_mode_respects_non_collapsible() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Single, false, SharedString::from("one"));

        assert_eq!(expanded.len(), 1);
        assert!(events.is_empty());
    }

    #[test]
    fn multiple_mode_toggles_independently() {
        let mut expanded = std::collections::HashSet::from([SharedString::from("one")]);

        let events = apply_toggle(&mut expanded, AccordionSelectionMode::Multiple, true, SharedString::from("three"));

        assert_eq!(expanded.len(), 2);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], super::AccordionEvent::ExpandedChanged { expanded: true, .. }));
    }

    /// Mirrors [`AccordionControl::handle_item_hover`] index bookkeeping.
    fn apply_accordion_item_hover(hovered_item_index: &mut Option<usize>, index: usize, hovered: bool) {
        if hovered {
            if *hovered_item_index != Some(index) {
                *hovered_item_index = Some(index);
            }
        } else if *hovered_item_index == Some(index) {
            *hovered_item_index = None;
        }
    }

    #[test]
    fn hover_false_only_clears_matching_index() {
        let mut hovered = Some(1usize);

        apply_accordion_item_hover(&mut hovered, 0, false);
        assert_eq!(hovered, Some(1));

        apply_accordion_item_hover(&mut hovered, 1, false);
        assert_eq!(hovered, None);

        apply_accordion_item_hover(&mut hovered, 2, true);
        assert_eq!(hovered, Some(2));
    }
}
