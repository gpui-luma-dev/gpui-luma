use gpui::Background;

/// Bar fill, independent of data and grouping. Supports native GPUI backgrounds.
#[derive(Clone, Copy, Debug)]
pub struct BarPresentation {
    pub fill: Background,
}
impl BarPresentation {
    pub fn new(fill: impl Into<Background>) -> Self {
        Self { fill: fill.into() }
    }
}

/// Label anchor relative to a bar's own baseline and endpoint, in every direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BarLabelPlacement {
    InsideStart,
    InsideEnd,
    Center,
    #[default]
    OutsideEnd,
}

/// One bar label. A callback may supply both a custom category label and a value.
#[derive(Clone, Debug)]
pub struct BarLabel {
    pub text: gpui::SharedString,
    pub placement: BarLabelPlacement,
    pub text_style: super::TextPresentation,
    pub gap: gpui::Pixels,
}
impl BarLabel {
    pub fn new(text: impl Into<gpui::SharedString>, text_style: super::TextPresentation) -> Self {
        Self { text: text.into(), placement: BarLabelPlacement::OutsideEnd, text_style, gap: gpui::px(8.0) }
    }
    pub fn placement(mut self, placement: BarLabelPlacement) -> Self {
        self.placement = placement;
        self
    }
    pub fn gap(mut self, gap: gpui::Pixels) -> Self {
        self.gap = gap;
        self
    }
}
