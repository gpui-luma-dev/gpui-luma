use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, anchored, deferred, div, point, prelude::*, px};

use crate::controls::selector_panel::SelectorItemsPanelAppearance;

use super::behavior::SelectionItem;
use super::item_template::SearchSelectorItemTemplate;

pub struct SearchSelectorPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a SearchSelectorItemTemplate<SelectionItem>>,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_appearance: SelectorItemsPanelAppearance,
    pub search_content: Option<AnyElement>,
    pub list_content: AnyElement,
}

pub trait SearchSelectorPanelTemplate: Send + Sync {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, cx: &mut App) -> AnyElement;
}

pub struct DefaultSearchSelectorPanelTemplate;

pub fn default_search_selector_panel_template() -> Arc<dyn SearchSelectorPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SearchSelectorPanelTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultSearchSelectorPanelTemplate)).clone()
}

impl SearchSelectorPanelTemplate for DefaultSearchSelectorPanelTemplate {
    fn render(&self, model: SearchSelectorPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
        let appearance = model.popup_appearance;

        let panel_content = div()
            .id(format!("{}-panel-content", model.id))
            .w_full()
            .when_some(model.search_content, |panel, search| {
                panel.child(div().p(px(8.0)).child(search)).child(div().h(px(1.0)).bg(appearance.border))
            })
            .child(model.list_content);

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
                            .child(panel_content),
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
            .child(panel_content)
            .into_any_element()
    }
}
