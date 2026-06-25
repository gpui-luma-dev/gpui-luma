mod control;
mod model;
mod template;
mod theme;

pub use control::DialogControl;
pub use model::{
    DialogBuilder, DialogDismissPolicy, DialogElementRenderer, DialogEvent, DialogMode, DialogModel, DialogPosition,
    DialogRenderModel, new,
};
pub use template::{
    DialogTemplate, DialogTemplateHandlers, DialogTemplateParts, ThemedDialogTemplate, default_dialog_template,
};
pub use theme::{DefaultDialogTheme, DialogLook, DialogTheme, default_dialog_theme};

use gpui::{Entity, SharedString};

pub type Dialog = Entity<DialogControl>;

pub fn dialog(id: impl Into<SharedString>) -> DialogBuilder {
    DialogBuilder::new(id)
}
