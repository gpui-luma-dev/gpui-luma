mod control;
mod model;
mod state;
mod template;

pub use control::{TextArea, TextAreaEvent};
pub use model::{TextAreaBuilder, TextAreaModel, TextAreaRenderModel, Validator};
pub use state::TextAreaState;
pub use template::{
    TextAreaClickHandler, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaMouseDownHandler, TextAreaTemplate,
    TextAreaTemplateHandlers, ThemedTextAreaTemplate, default_textarea_template,
};
