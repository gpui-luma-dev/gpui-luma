mod behavior;
mod control;
mod model;
mod template;
mod text_selection;
mod theme;

pub use behavior::SelectionItem;
pub use control::{AutocompleteTextBox, AutocompleteTextBoxControl, AutocompleteTextBoxEvent};
pub use model::{AutocompleteTextBoxBuilder, AutocompleteTextBoxModel, new};

pub type Autocomplete = AutocompleteTextBox;
pub type AutocompleteControl = AutocompleteTextBoxControl;
pub type AutocompleteEvent = AutocompleteTextBoxEvent;
pub type AutocompleteBuilder = AutocompleteTextBoxBuilder;
pub type AutocompleteModel = AutocompleteTextBoxModel;
pub use template::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplate, AutocompleteItemsTemplateHandlers,
    AutocompleteItemsTemplateModifier, AutocompleteTextBoxRenderModel, AutocompleteTextBoxTemplate,
    AutocompleteTextBoxTemplateHandlers, AutocompleteTextBoxTemplateModifier, DefaultAutocompleteItemsTemplate,
    DefaultAutocompleteTextBoxTemplate, default_autocomplete_items_template, default_autocomplete_textbox_template,
};
pub use theme::{
    DefaultAutocompleteTextBoxTheme, AutocompleteTextBoxLook, AutocompleteTextBoxTheme,
    default_autocomplete_textbox_theme,
};

pub type AutocompleteLook = AutocompleteTextBoxLook;
pub use AutocompleteTextBoxTheme as AutocompleteTheme;
