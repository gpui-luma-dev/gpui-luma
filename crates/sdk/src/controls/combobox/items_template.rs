use std::sync::{Arc, OnceLock};

use gpui::{SharedString, Stateful, div, prelude::*, px};

use crate::controls::selector_panel::{
    SelectorItem, SelectorItemsPanelAppearance, SelectorPanelClickHandler, SelectorPanelHoverHandler,
};

pub struct ComboBoxItemsRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectorItem],
    pub appearance: SelectorItemsPanelAppearance,
    pub highlighted_index: Option<usize>,
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
    ) -> Stateful<gpui::Div> {
        let ComboBoxItemsTemplateHandlers { item_hovers, item_clicks } = handlers;
        let appearance = model.appearance.clone();

        let mut root = div()
            .id(format!("{}-rows", model.id))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(appearance.min_width))
            .p(px(appearance.padding))
            .bg(appearance.background)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .shadow(appearance.shadow.clone())
            .occlude();
        let mut clicks = item_clicks.into_iter();

        for (index, (item, hover)) in model.items.iter().zip(item_hovers).enumerate() {
            let mut row = div()
                .id(format!("{}-row-{}", model.id, index))
                .flex()
                .items_center()
                .min_h(px(appearance.item_height))
                .px(px(appearance.item_padding_x))
                .rounded(px(appearance.item_radius))
                .text_color(appearance.foreground)
                .text_size(px(appearance.item_typography.size))
                .line_height(px(appearance.item_typography.line_height))
                .font_weight(appearance.item_typography.weight)
                .child(item.label_text().clone());

            row = row.cursor_pointer().on_hover(hover).hover({
                let hover_background = appearance.item_hover_background;
                move |style| style.bg(hover_background)
            });

            if model.highlighted_index.is_some_and(|active| active == index) {
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
