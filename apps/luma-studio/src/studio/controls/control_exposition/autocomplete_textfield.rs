use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::autocomplete::{AutocompleteTextBox, AutocompleteTextBoxEvent, SelectionItem};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::toggle::{Toggle, ToggleEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::autocomplete_inspector_adapter::{AutocompleteInspectorAdapter, AUTOCOMPLETE_INSPECTOR_SPEC};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::AutocompleteThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct AutocompleteTextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: AutocompleteTextBox,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<AutocompleteExpositionLeftPane>,
    selection_required: bool,
    has_selection: bool,
    theme_inspector: Entity<AutocompleteThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct AutocompleteExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: AutocompleteTextBox,
    event_stream: Entity<ControlEventStream>,
    selection_required_toggle: Toggle,
}

impl AutocompleteExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.selection_required_toggle.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for AutocompleteExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .max_w(px(760.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(
                    div()
                        .w(px(320.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(self.selection_required_toggle.clone())
                        .child(self.preview.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-autocomplete-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl AutocompleteTextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("autocomplete-textfield").expect("autocomplete-textfield catalog entry");
        let preview = look
            .autocomplete("controls-doc-autocomplete", autocomplete_demo_items())
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .invalid(false)
            .spawn(cx);
        let selection_required_toggle = look
            .outline_toggle("controls-doc-autocomplete-selection-required")
            .with_data(false)
            .content(|_, _| div().child("Selection Required").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-autocomplete-event-log",
                "Type in the field and pick items; AutocompleteTextBoxEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| AutocompleteExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
            selection_required_toggle: selection_required_toggle.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-autocomplete-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &AUTOCOMPLETE_INSPECTOR_SPEC,
            AutocompleteInspectorAdapter::shared(),
        );

        let subscription = cx.subscribe(&preview, {
            let event_stream = event_stream.clone();
            move |this, _, event: &AutocompleteTextBoxEvent, cx| {
                let line = format_autocomplete_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
                match event {
                    AutocompleteTextBoxEvent::Select { .. } | AutocompleteTextBoxEvent::Complete { .. } => {
                        this.has_selection = true;
                    }
                    AutocompleteTextBoxEvent::Clear => this.has_selection = false,
                    _ => {}
                }
                this.sync_required_validation(cx);
            }
        });
        let required_subscription = cx.subscribe(&selection_required_toggle, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected } = event {
                this.selection_required = *selected;
                this.sync_required_validation(cx);
            }
        });

        Self {
            look,
            entry,
            preview,
            event_stream,
            left_pane,
            selection_required: false,
            has_selection: false,
            theme_inspector,
            inspector_split,
            _subscriptions: vec![subscription, required_subscription],
        }
    }

    fn sync_required_validation(&mut self, cx: &mut Context<Self>) {
        let invalid = self.selection_required && !self.has_selection;
        self.preview.update(cx, |preview, cx| preview.set_invalid(invalid, cx));
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for AutocompleteTextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-autocomplete-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn autocomplete_demo_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("alabama", "Alabama"),
        SelectionItem::new("alaska", "Alaska"),
        SelectionItem::new("arizona", "Arizona"),
        SelectionItem::new("arkansas", "Arkansas"),
        SelectionItem::new("california", "California"),
        SelectionItem::new("colorado", "Colorado"),
        SelectionItem::new("connecticut", "Connecticut"),
        SelectionItem::new("delaware", "Delaware"),
        SelectionItem::new("florida", "Florida"),
        SelectionItem::new("georgia", "Georgia"),
    ]
}

fn format_autocomplete_event(event: &AutocompleteTextBoxEvent) -> String {
    match event {
        AutocompleteTextBoxEvent::Change { query } => {
            format!("AutocompleteTextBoxEvent::Change {{ query: \"{query}\" }}")
        }
        AutocompleteTextBoxEvent::Select { item_id, label } => {
            format!("AutocompleteTextBoxEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteTextBoxEvent::Complete { item_id, label } => {
            format!("AutocompleteTextBoxEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteTextBoxEvent::Clear => "AutocompleteTextBoxEvent::Clear".to_string(),
        AutocompleteTextBoxEvent::OpenChanged { open } => {
            format!("AutocompleteTextBoxEvent::OpenChanged {{ open: {open} }}")
        }
        AutocompleteTextBoxEvent::Dismiss => "AutocompleteTextBoxEvent::Dismiss".to_string(),
        AutocompleteTextBoxEvent::FocusChanged { focused } => {
            format!("AutocompleteTextBoxEvent::FocusChanged {{ focused: {focused} }}")
        }
        _ => "AutocompleteTextBoxEvent::(unknown)".to_string(),
    }
}
