//! SearchSelector supports layered template customization:
//! - control template (`SearchSelectorTemplate`)
//! - panel template (`SearchSelectorPanelTemplate`)
//! - item template (`SearchSelectorItemTemplate`)
//!
//! For new integrations, prefer `panel_template(...)` + `with_item_template(...)`
//! over the transitional `items_template(...)` hook.

mod behavior;
mod control;
mod item_template;
mod model;
mod panel_template;
mod template;
mod text_selection;

pub use behavior::SelectionItem;
pub use control::{SearchSelector, SearchSelectorControl, SearchSelectorEvent};
pub use item_template::{
    SearchSelectorItemRenderModel, SearchSelectorItemTemplate, SearchSelectorItemTemplateModifier,
    item_template_with_modifier, make_search_selector_item_template,
};
pub use model::{SearchSelectorBuilder, SearchSelectorModel, new};
pub use panel_template::{
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, SearchSelectorPanelTemplateModifier,
    default_search_selector_panel_template, panel_template_with_modifier,
};
pub use template::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers, SearchSelectorTemplateModifier,
    default_search_selector_items_template, default_search_selector_template, template_with_modifier,
};
#[allow(deprecated)]
pub use template::render_popup_rows;
