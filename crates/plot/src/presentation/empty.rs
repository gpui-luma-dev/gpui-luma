use gpui::{Hsla, SharedString};

/// Copy and color shown when a plot selects no finite samples.
#[derive(Clone, Debug)]
pub struct EmptyStatePresentation {
    pub message: SharedString,
    pub text_color: Hsla,
}

impl EmptyStatePresentation {
    pub fn new(message: impl Into<SharedString>, text_color: Hsla) -> Self {
        Self { message: message.into(), text_color }
    }
}
