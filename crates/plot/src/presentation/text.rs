use gpui::{Hsla, Pixels, SharedString, px};

/// Resolved typography for plot text, independent of its content.
#[derive(Clone, Debug)]
pub struct TextPresentation {
    pub color: Hsla,
    pub font_family: SharedString,
    pub font_size: Pixels,
    pub line_height: Pixels,
}

impl TextPresentation {
    pub fn new(color: Hsla) -> Self {
        Self { color, font_family: "System UI".into(), font_size: px(11.0), line_height: px(16.0) }
    }
}
