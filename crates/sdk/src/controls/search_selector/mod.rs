mod behavior;
mod control;
mod model;
mod template;
mod text_selection;

pub use behavior::SelectionItem;
pub use control::{SearchSelector, SearchSelectorControl, SearchSelectorEvent};
pub use crate::controls::autocomplete::COMBOBOX_THEME_USAGE as SEARCH_SELECTOR_THEME_USAGE;
pub use model::{SearchSelectorBuilder, SearchSelectorModel, new};
pub use template::{
    SearchSelectorRenderModel, SearchSelectorTemplate, SearchSelectorTemplateHandlers,
    default_search_selector_template, render_popup_rows,
};
