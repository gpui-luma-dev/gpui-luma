use std::sync::Arc;

use gpui::{AnyElement, App, SharedString};
use gpui::prelude::*;

use gpui_luma::controls::combobox::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, ComboBoxPanelRenderModel,
    ComboBoxPanelTemplate, SelectionItem as ComboBoxSelectionItem,
};
use gpui_luma::controls::search_selector::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, SelectionItem as SearchSelectorSelectionItem,
};
use gpui_luma::controls::selector_panel::{
    SelectorItem as SelectorPanelItem, SelectorItemsPanelAppearance, SelectorPanelClickHandler,
    SelectorPanelHoverHandler,
};

pub(in crate::gallery) fn render_combobox_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    appearance: SelectorItemsPanelAppearance,
    items_template: &Arc<dyn ComboBoxItemsTemplate>,
    panel_template: &Arc<dyn ComboBoxPanelTemplate>,
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
    cx: &mut App,
) -> AnyElement {
    let combobox_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            ComboBoxSelectionItem::new(format!("combobox-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..combobox_items.len()).collect::<Vec<_>>();

    let list = items_template.render(
        &ComboBoxItemsRenderModel {
            menu_id: popup_id,
            combobox_id: popup_id,
            items: &combobox_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            appearance: appearance.clone(),
        },
        ComboBoxItemsTemplateHandlers { item_hovers, item_clicks },
        cx,
    );

    panel_template.render(
        ComboBoxPanelRenderModel {
            id: popup_id,
            items: &combobox_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_appearance: appearance,
            list_content: list.into_any_element(),
        },
        cx,
    )
}

pub(in crate::gallery) fn render_search_selector_popup_preview_from_templates(
    popup_id: &SharedString,
    items: &[SelectorPanelItem],
    appearance: SelectorItemsPanelAppearance,
    items_template: &Arc<dyn SearchSelectorItemsTemplate>,
    panel_template: &Arc<dyn SearchSelectorPanelTemplate>,
    search_content: AnyElement,
    item_hovers: Vec<SelectorPanelHoverHandler>,
    item_clicks: Vec<SelectorPanelClickHandler>,
    cx: &mut App,
) -> AnyElement {
    let search_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            SearchSelectorSelectionItem::new(format!("search-selector-preview-item-{index}"), item.label_text().clone())
        })
        .collect::<Vec<_>>();
    let visible_indices = (0..search_items.len()).collect::<Vec<_>>();

    let rows = items_template
        .render(
            &SearchSelectorItemsRenderModel {
                menu_id: popup_id,
                search_selector_id: popup_id,
                items: &search_items,
                visible_indices: &visible_indices,
                selected_source_index: None,
                active_visible_index: Some(0),
                open: true,
                enabled: true,
                item_template: None,
                appearance: appearance.clone(),
            },
            SearchSelectorItemsTemplateHandlers { item_hovers, item_clicks },
            cx,
        )
        .into_any_element();

    panel_template.render(
        SearchSelectorPanelRenderModel {
            id: popup_id,
            items: &search_items,
            visible_indices: &visible_indices,
            selected_source_index: None,
            active_visible_index: Some(0),
            open: true,
            enabled: true,
            item_template: None,
            popup_bounds: None,
            popup_appearance: appearance,
            search_content: Some(search_content),
            list_content: rows,
        },
        cx,
    )
}
