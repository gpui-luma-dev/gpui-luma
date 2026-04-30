mod control;
mod model;
mod state;
mod template;

pub use control::TextFieldEvent;
pub use model::{TextFieldBuilder, TextFieldModel, TextFieldRenderModel, Validator};
pub use state::TextFieldState;
pub use template::{
    TextFieldClickHandler, TextFieldHoverHandler, TextFieldKeyDownHandler, TextFieldMouseDownHandler,
    TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplate, TextFieldTemplateHandlers,
    ThemedTextFieldTemplate, default_textfield_template,
};

use gpui::{Entity, SharedString};

use self::control::TextFieldControl;

pub type TextField = Entity<TextFieldControl>;

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    TextFieldBuilder::new(id)
}
