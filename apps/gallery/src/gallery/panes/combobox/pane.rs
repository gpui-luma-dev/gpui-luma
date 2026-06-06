use std::sync::Arc;

use gpui::{AnyElement, App, Context, Subscription, anchored, deferred, div, point, prelude::*, px};
use gpui_luma::controls::combobox::{
    ComboBox, ComboBoxEvent, ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers,
    ComboBoxPanelRenderModel, ComboBoxPanelTemplate, SelectionItem, TypingPolicy,
};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::theme::{ControlSize};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_descriptions, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ComboBoxPane {
    strict_combobox: ComboBox,
}

impl ComboBoxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let strict_combobox = look
            .combobox("gallery-combobox-strict", combobox_demo_items())
            .items_template(Arc::new(GalleryComboboxItemsTemplate::new(look.clone())))
            .panel_template(Arc::new(GalleryComboboxPanelTemplate::new(look.clone())))
            .placeholder("Strict mode (exact match only)…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(true)
            .spawn(cx);

        Self { strict_combobox }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.strict_combobox, |app, _, _event: &ComboBoxEvent, cx| {
            app.panes.combobox.notify_controls(cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook) -> AnyElement {
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
                        .text_color(look.chrome().muted_text)
                        .child("Strict typing policy + down arrow"),
                )
                .child(self.strict_combobox.clone())
                .into_any_element(),
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.strict_combobox, cx);
    }
}

#[derive(Clone)]
struct GalleryComboboxItemsTemplate {
    look: Arc<ShadcnLook>,
}

impl GalleryComboboxItemsTemplate {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl ComboBoxItemsTemplate for GalleryComboboxItemsTemplate {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        let appearance = self.look.selector_items_panel_appearance(ControlSize::Md);
        let ComboBoxItemsTemplateHandlers { item_hovers, item_clicks } = handlers;

        let mut root = div()
            .id(format!("{}-rows", model.menu_id))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(appearance.min_width))
            .p(px(appearance.padding));
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let selected = model.selected_source_index == Some(source_index);
            let active = model.active_visible_index == Some(visible_index);
            let content = if let Some(item_template) = model.item_template {
                item_template(
                    &gpui_luma::controls::combobox::ComboBoxItemRenderModel {
                        combobox_id: model.combobox_id,
                        item,
                        source_index,
                        visible_index,
                        selected,
                        active,
                        open: model.open,
                        enabled: model.enabled,
                    },
                    cx,
                )
            } else {
                div().flex_1().child(item.label.clone()).into_any_element()
            };

            let mut row = div()
                .id(format!("{}-row-{}", model.menu_id, visible_index))
                .flex()
                .items_center()
                .min_h(px(appearance.item_height))
                .px(px(appearance.item_padding_x))
                .rounded(px(appearance.item_radius))
                .text_color(appearance.foreground)
                .text_size(px(appearance.item_typography.size))
                .line_height(px(appearance.item_typography.line_height))
                .font_weight(appearance.item_typography.weight)
                .child(content);

            row = row.cursor_pointer().on_hover(hover).hover({
                let hover_background = appearance.item_hover_background;
                let hover_foreground = appearance.item_hover_foreground;
                move |style| style.bg(hover_background).text_color(hover_foreground)
            });

            if active {
                row = row.bg(appearance.item_hover_background).text_color(appearance.item_hover_foreground);
            }

            if let Some(click) = clicks.next() {
                row = row.on_click(click);
            }

            root = root.child(row);
        }

        root
    }
}

#[derive(Clone)]
struct GalleryComboboxPanelTemplate {
    look: Arc<ShadcnLook>,
}

impl GalleryComboboxPanelTemplate {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl ComboBoxPanelTemplate for GalleryComboboxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
        let appearance = self.look.selector_items_panel_appearance(ControlSize::Md);

        if let Some(bounds) = model.popup_bounds {
            return deferred(
                anchored()
                    .snap_to_window_with_margin(px(8.0))
                    .anchor(gpui::Corner::TopLeft)
                    .position(point(bounds.left(), bounds.bottom()))
                    .offset(point(px(0.0), px(4.0)))
                    .child(
                        div()
                            .id(format!("{}-popup-shell", model.id))
                            .w(bounds.size.width)
                            .bg(appearance.background)
                            .border_1()
                            .border_color(appearance.border)
                            .rounded(px(appearance.radius))
                            .shadow(appearance.shadow)
                            .overflow_hidden()
                            .occlude()
                            .child(model.list_content),
                    ),
            )
            .with_priority(1)
            .into_any_element();
        }

        div()
            .id(format!("{}-panel", model.id))
            .w_full()
            .bg(appearance.background)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .shadow(appearance.shadow)
            .overflow_hidden()
            .occlude()
            .child(model.list_content)
            .into_any_element()
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
