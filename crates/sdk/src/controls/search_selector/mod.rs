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
    make_search_selector_item_template,
};
pub use model::{SearchSelectorBuilder, SearchSelectorModel, new};
pub use panel_template::{
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, SearchSelectorPanelTemplateModifier,
    default_search_selector_panel_template,
};
pub use template::{
    DefaultSearchSelectorItemsTemplate, SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate,
    SearchSelectorItemsTemplateHandlers, SearchSelectorItemsTemplateModifier, SearchSelectorRenderModel,
    SearchSelectorTemplate, SearchSelectorTemplateHandlers, SearchSelectorTemplateModifier,
    default_search_selector_items_template, default_search_selector_template,
    search_selector_items_template_with_modifier,
};
