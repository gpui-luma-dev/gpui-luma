use std::sync::Arc;

use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::autocomplete::{self, AutocompleteTextBox, AutocompleteTextBoxEvent, SelectionItem};
use gpui_luma::theme::RadixTheme;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_descriptions, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct AutocompleteTextFieldPane {
    autocomplete_textbox: AutocompleteTextBox,
}

impl AutocompleteTextFieldPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, radix_theme: Arc<RadixTheme>) -> Self {
        let demo_items = autocomplete_demo_items();
        let autocomplete_textbox = autocomplete::new("prototype-autocomplete", demo_items)
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .textfield_template(radix_theme.textfield_template())
            .scrollbar_template(radix_theme.scrollbar_template())
            .spawn(cx);

        Self { autocomplete_textbox }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(
            &self.autocomplete_textbox,
            |app, _, _event: &AutocompleteTextBoxEvent, cx| {
                app.panes.autocomplete_textfield.notify_controls(cx);
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, radix_theme: &RadixTheme) -> AnyElement {
        gallery_pane_with_usage_descriptions(
            "Autocomplete TextBox",
            Some("Autocomplete text box"),
            &["Autocomplete TextBox", "Floating Menu"],
            div()
                .w(px(240.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(self.autocomplete_textbox.clone())
                .into_any_element(),
            radix_theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.autocomplete_textbox, cx);
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
