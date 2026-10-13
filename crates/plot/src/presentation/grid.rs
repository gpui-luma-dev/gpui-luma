use gpui::{Hsla, px};

use super::Stroke;

/// Grid lines driven by scale ticks, independent of visible axis edges.
#[derive(Clone, Copy, Debug)]
pub struct GridPresentation {
    pub vertical: Option<Stroke>,
    pub horizontal: Option<Stroke>,
}

impl GridPresentation {
    pub fn new(color: Hsla) -> Self {
        let stroke = Stroke { color, width: px(1.0) };
        Self { vertical: Some(stroke), horizontal: Some(stroke) }
    }
}
