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
pub use crate::controls::autocomplete::COMBOBOX_THEME_USAGE as SEARCH_SELECTOR_THEME_USAGE;
pub use item_template::{SearchSelectorItemRenderModel, SearchSelectorItemTemplate, make_search_selector_item_template};
pub use model::{SearchSelectorBuilder, SearchSelectorModel, new};
pub use panel_template::{
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, default_search_selector_panel_template,
};
pub use template::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers,
    default_search_selector_items_template, default_search_selector_template,
};
#[allow(deprecated)]
pub use template::render_popup_rows;
