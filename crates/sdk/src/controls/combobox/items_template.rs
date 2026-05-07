use std::sync::{Arc, OnceLock};

use gpui::{App, SharedString, Stateful, div, prelude::*, px};

use super::behavior::SelectionItem;
use super::item_template::ComboBoxItemTemplate;
use crate::controls::selector_panel::{SelectorItemsPanelAppearance, SelectorPanelClickHandler, SelectorPanelHoverHandler};

pub struct ComboBoxItemsRenderModel<'a> {
    pub menu_id: &'a SharedString,
    pub combobox_id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a ComboBoxItemTemplate<SelectionItem>>,
    pub appearance: SelectorItemsPanelAppearance,
}

pub struct ComboBoxItemsTemplateHandlers {
    pub item_hovers: Vec<SelectorPanelHoverHandler>,
    pub item_clicks: Vec<SelectorPanelClickHandler>,
}

pub trait ComboBoxItemsTemplate: Send + Sync {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div>;
}

pub struct DefaultComboBoxItemsTemplate;

pub fn default_combobox_items_template() -> Arc<dyn ComboBoxItemsTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxItemsTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxItemsTemplate)).clone()
}

impl ComboBoxItemsTemplate for DefaultComboBoxItemsTemplate {
    fn render(
        &self,
        model: &ComboBoxItemsRenderModel<'_>,
        handlers: ComboBoxItemsTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<gpui::Div> {
        let ComboBoxItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let appearance = model.appearance.clone();

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
                    &super::item_template::ComboBoxItemRenderModel {
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
                move |style| style.bg(hover_background)
            });

            if active {
                row = row.bg(appearance.item_hover_background);
            }

            if let Some(click) = clicks.next() {
                row = row.on_click(click);
            }

            root = root.child(row);
        }

        root
    }
}
