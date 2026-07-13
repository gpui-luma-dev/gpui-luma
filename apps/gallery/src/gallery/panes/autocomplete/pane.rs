use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::autocomplete::{AutocompleteTextBox, AutocompleteTextBoxEvent, SelectionItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_autocomplete_inspect_tree;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

#[derive(Clone)]
pub(in crate::gallery) struct AutocompleteTextFieldPane {
    autocomplete_textbox: AutocompleteTextBox,
    inspector: Entity<ColorInspectorShell>,
}

impl AutocompleteTextFieldPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "autocomplete-inspector-tree",
            look.clone(),
            build_autocomplete_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "autocomplete-inspector",
                "autocomplete-inspector-split",
                "autocomplete-inspector-detail",
                build_autocomplete_inspect_tree,
                cx,
            )
        });

        let autocomplete_textbox = look
            .autocomplete("prototype-autocomplete", autocomplete_demo_items())
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);

        Self { autocomplete_textbox, inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(
            &self.autocomplete_textbox,
            |app, _, _event: &AutocompleteTextBoxEvent, cx| {
                app.panes.autocomplete_textfield.notify_controls(cx);
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        gallery_pane_with_inspector(
            "autocomplete-textfield",
            "Autocomplete TextBox",
            div()
                .w(px(240.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(self.autocomplete_textbox.clone())
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.autocomplete_textbox, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

fn autocomplete_demo_items() -> Vec<SelectionItem> {
    let mut items = vec![
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
        SelectionItem::new("hawaii", "Hawaii"),
        SelectionItem::new("idaho", "Idaho"),
        SelectionItem::new("illinois", "Illinois"),
        SelectionItem::new("indiana", "Indiana"),
        SelectionItem::new("iowa", "Iowa"),
        SelectionItem::new("kansas", "Kansas"),
        SelectionItem::new("kentucky", "Kentucky"),
    ];

    items.extend((1..=110).map(|index| {
        SelectionItem::new(format!("autocomplete-demo-{index:03}"), format!("Autocomplete Demo Item {index:03}"))
    }));

    items
}
