use gpui::SharedString;
use gpui_luma::controls::textfield::{self, TextField, TextFieldBuilder, TextFieldEvent};

pub type TextSelection = TextField;

#[derive(Clone, Debug)]
pub enum TextSelectionEvent {
    Change { value: String },
    Submit { value: String },
    FocusEnter,
    FocusLeave,
}

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    textfield::new(id)
}

pub fn map_event(event: &TextFieldEvent) -> TextSelectionEvent {
    match event {
        TextFieldEvent::Change { value } => TextSelectionEvent::Change { value: value.clone() },
        TextFieldEvent::Submit { value } => TextSelectionEvent::Submit { value: value.clone() },
        TextFieldEvent::Focus => TextSelectionEvent::FocusEnter,
        TextFieldEvent::Blur => TextSelectionEvent::FocusLeave,
    }
}
