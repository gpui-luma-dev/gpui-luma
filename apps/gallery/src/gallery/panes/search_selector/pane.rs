use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Subscription, div, prelude::*, px};
use gpui_luma::controls::search_selector::{SearchSelector, SearchSelectorEvent, SelectionItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_search_selector_inspect_tree;
use crate::gallery::panes::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use crate::gallery::panes::shared::{gallery_pane_with_inspector, notify_entity, InspectorToggleRegistry};

#[derive(Clone)]
pub(in crate::gallery) struct SearchSelectorPane {
    selector: SearchSelector,
    inspector: Entity<ColorInspectorShell>,
}

impl SearchSelectorPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree = spawn_color_inspector_tree(
            "search-selector-inspector-tree",
            look.clone(),
            build_search_selector_inspect_tree,
            cx,
        );
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "search-selector-inspector",
                "search-selector-inspector-split",
                "search-selector-inspector-detail",
                build_search_selector_inspect_tree,
                cx,
            )
        });

        let selector = look
            .search_selector("gallery-search-selector", search_selector_demo_items())
            .placeholder("Choose a state…")
            .search_placeholder("Selection search")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);

        Self { selector, inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.selector, |app, _, _event: &SearchSelectorEvent, cx| {
            app.panes.search_selector.notify_controls(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        gallery_pane_with_inspector(
            "search-selector",
            "SearchSelector",
            div()
                .w(px(280.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(look.chrome().muted_text)
                        .child("Read-only trigger + popup search input"),
                )
                .child(self.selector.clone())
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.selector, cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }
}

fn search_selector_demo_items() -> Vec<SelectionItem> {
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
        SelectionItem::new("hawaii", "Hawaii"),
        SelectionItem::new("idaho", "Idaho"),
        SelectionItem::new("illinois", "Illinois"),
        SelectionItem::new("indiana", "Indiana"),
        SelectionItem::new("iowa", "Iowa"),
        SelectionItem::new("kansas", "Kansas"),
        SelectionItem::new("kentucky", "Kentucky"),
    ]
}
