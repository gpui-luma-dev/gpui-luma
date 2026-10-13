use gpui::{Hsla, Pixels, px};

/// Visual treatment of a line. Non-positive or non-finite widths hide its stroke.
#[derive(Clone, Copy, Debug)]
pub struct Stroke {
    pub color: Hsla,
    pub width: Pixels,
}

/// Which observations have visible markers. Gaps and interpolated curve positions
/// never create extra observations; decimation marks retained observations only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerVisibility {
    #[default]
    Always,
    Hover,
    /// Select observations sharing a coordinate anywhere inside the plot.
    /// Currently selects X, independent of pointer Y.
    HoverSlice,
}

/// Optional circular sample markers, independent of the line stroke.
/// Non-positive or non-finite radii hide markers.
#[derive(Clone, Copy, Debug)]
pub struct MarkerPresentation {
    pub fill: Hsla,
    pub radius: Pixels,
    pub visibility: MarkerVisibility,
}

impl MarkerPresentation {
    pub fn new(fill: Hsla, radius: Pixels) -> Self {
        Self { fill, radius, visibility: MarkerVisibility::Always }
    }
    /// Show all retained observations, or only the chart's nearest hovered point.
    pub fn visibility(mut self, visibility: MarkerVisibility) -> Self {
        self.visibility = visibility;
        self
    }
}

/// Focused line visuals; no data, axes, layout, or interaction options.
#[derive(Clone, Copy, Debug)]
pub struct LinePresentation {
    pub stroke: Stroke,
    pub markers: Option<MarkerPresentation>,
}

impl LinePresentation {
    /// A two-pixel stroke without markers. The caller supplies the resolved color.
    pub fn new(color: Hsla) -> Self {
        Self { stroke: Stroke { color, width: px(2.0) }, markers: None }
    }
}
