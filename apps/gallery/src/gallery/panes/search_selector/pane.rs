use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::search_selector::{SearchSelector, SearchSelectorEvent, SelectionItem};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_descriptions, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct SearchSelectorPane {
    selector: SearchSelector,
}

impl SearchSelectorPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let selector = look
            .search_selector("gallery-search-selector", search_selector_demo_items())
            .placeholder("Choose a state…")
            .search_placeholder("Selection search")
            .full_width(true)
            .clean_on_escape(true)
            .spawn(cx);

        Self { selector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.selector, |app, _, _event: &SearchSelectorEvent, cx| {
            app.panes.search_selector.notify_controls(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
        gallery_pane_with_usage_descriptions(
            "SearchSelector",
            Some(
                "Read-only selector with a search icon trigger. Focus opens a popup containing an inline search field.",
            ),
            &["Autocomplete TextBox", "Floating Menu"],
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
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.selector, cx);
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
