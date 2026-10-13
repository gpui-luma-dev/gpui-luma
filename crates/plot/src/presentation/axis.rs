use gpui::{Hsla, Pixels, SharedString, px};

use super::{Stroke, TextPresentation};

/// The plot edge presenting a scale. Top/bottom use X; left/right use Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisSide {
    Top,
    Bottom,
    Left,
    Right,
}

impl AxisSide {
    pub(crate) fn is_horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// A visual axis bound to a Cartesian scale by its edge.
///
/// Opposing axes share one scale. This component has no data or numeric domain.
/// Vertical-axis titles are horizontal headings above their tick-label column.
#[derive(Clone, Debug)]
pub struct Axis {
    pub side: AxisSide,
    pub title: Option<SharedString>,
    pub presentation: AxisPresentation,
}

impl Axis {
    pub fn new(side: AxisSide, presentation: AxisPresentation) -> Self {
        Self { side, title: None, presentation }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Independent appearance and spacing for one axis. Grid treatment is separate.
#[derive(Clone, Debug)]
pub struct AxisPresentation {
    pub stroke: Stroke,
    pub tick_labels: TextPresentation,
    pub title: TextPresentation,
    /// Minimum reserved label band; measured text can enlarge it.
    pub gutter: Pixels,
    pub tick_length: Pixels,
    pub label_gap: Pixels,
    /// Fixed tick-label slot width. None measures each label as before.
    /// Positive finite widths keep layout independent of changing tick text.
    pub label_width: Option<Pixels>,
    /// Text alignment within fixed-width slots; ignored for measured labels.
    pub label_align: gpui::TextAlign,
    /// Keep horizontal endpoint slots inside their ticks, with `label_gap` padding.
    /// Endpoint text aligns inward; interior text centers. Requires `label_width`.
    pub inset_end_labels: bool,
}

impl AxisPresentation {
    pub fn new(color: Hsla) -> Self {
        Self {
            stroke: Stroke { color, width: px(1.0) },
            tick_labels: TextPresentation::new(color),
            title: TextPresentation::new(color),
            gutter: px(32.0),
            tick_length: px(4.0),
            label_gap: px(4.0),
            label_width: None,
            label_align: gpui::TextAlign::Left,
            inset_end_labels: false,
        }
    }

    pub fn tick_labels(mut self, tick_labels: TextPresentation) -> Self {
        self.tick_labels = tick_labels;
        self
    }
}
