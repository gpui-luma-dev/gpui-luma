use std::sync::{Arc, OnceLock};

use gpui::{App, SharedString, Stateful, div, prelude::*, px};

use super::behavior::SelectionItem;
use super::item_template::ComboBoxItemTemplate;
use crate::controls::selector_panel::{SelectorItemsPanelLook, SelectorPanelClickHandler, SelectorPanelHoverHandler};

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
    pub look: SelectorItemsPanelLook,
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
        let look = model.look.clone();

        let mut root = div()
            .id(format!("{}-rows", model.menu_id))
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .p(px(look.padding));
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let enabled_item = model.enabled && item.enabled;
            let color = if enabled_item {
                look.foreground
            } else {
                look.item_disabled_foreground
            };
            let selected = model.selected_source_index == Some(source_index);
            let active = enabled_item && model.active_visible_index == Some(visible_index);
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
                        enabled: enabled_item,
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
                .min_h(px(look.item_height))
                .px(px(look.item_padding_x))
                .rounded(px(look.item_radius))
                .text_color(color)
                .text_size(px(look.item_typography.size))
                .line_height(px(look.item_typography.line_height))
                .font_weight(look.item_typography.weight)
                .child(content);

            if enabled_item {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = look.item_hover_background;
                    let hover_foreground = look.item_hover_foreground;
                    move |style| style.bg(hover_background).text_color(hover_foreground)
                });

                if active {
                    row = row.bg(look.item_hover_background).text_color(look.item_hover_foreground);
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = row.opacity(0.56);
            }

            root = root.child(row);
        }

        root
    }
}
