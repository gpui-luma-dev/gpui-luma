use gpui::{AnyElement, Context, Subscription, div, prelude::*, px};
use gpui_luma::controls::combobox::{self, ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_descriptions, notify_entity};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone)]
pub(in crate::gallery) struct ComboBoxPane {
    strict_combobox: ComboBox,
}

impl ComboBoxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let theme_clone = theme.clone();
        let strict_combobox = combobox::new("gallery-combobox-strict", combobox_demo_items())
            .placeholder("Strict mode (exact match only)…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(true)
            .textfield_template(theme_clone.textfield_template())
            .scrollbar_template(theme_clone.scrollbar_template())
            .spawn(cx);

        Self { strict_combobox }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.strict_combobox, |app, _, _event: &ComboBoxEvent, cx| {
            app.panes.combobox.notify_controls(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage_descriptions(
            "ComboBox",
            Some("Select-oriented input with dropdown trigger, open/close toggle, and strict typing policy."),
            &["ComboBox", "Floating Menu"],
            div()
                .w(px(240.0))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(theme.chrome().muted_text)
                        .child("Strict typing policy + down arrow"),
                )
                .child(self.strict_combobox.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.strict_combobox, cx);
    }
}

fn combobox_demo_items() -> Vec<SelectionItem> {
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
