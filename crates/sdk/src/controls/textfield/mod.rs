mod control;
mod model;
mod state;
mod template;
mod theme;

pub use control::TextFieldEvent;
pub use model::{TextFieldLookOverride, TextFieldBuilder, TextFieldModel, TextFieldRenderModel, Validator};
pub use state::TextFieldState;
pub use template::{
    TextFieldClickHandler, TextFieldHoverHandler, TextFieldKeyDownHandler, TextFieldMouseDownHandler,
    TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplate, TextFieldTemplateHandlers,
    ThemedTextFieldTemplate, default_textfield_template,
};
pub use theme::{
    DefaultTextFieldTheme, TextFieldLook, TextFieldPalette, TextFieldTheme, TextFieldVariant,
    default_textfield_theme,
};
#[allow(unused_imports)]
pub(crate) use theme::compose_textfield_appearance;

use gpui::{Entity, SharedString};

use self::control::TextFieldControl;

pub type TextField = Entity<TextFieldControl>;

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    TextFieldBuilder::new(id)
}
