//! Focused visual contracts, independent of data and view state.

mod axis;
mod cartesian;
mod empty;
mod grid;
mod text;
mod legend;
mod line;
mod metrics;

pub use axis::{Axis, AxisPresentation, AxisSide};
pub use grid::GridPresentation;
pub use text::TextPresentation;
pub use cartesian::CartesianPresentation;
pub use empty::EmptyStatePresentation;
pub use legend::{LegendPresentation, SwatchPresentation};
pub use line::{LinePresentation, MarkerPresentation, MarkerVisibility, Stroke};
pub use metrics::ChartMetrics;

mod area;
pub use area::AreaPresentation;

mod style;
pub use style::{CartesianStyle, CartesianStyleProvider, SeriesStyle};

/// Compatibility name for [CartesianStyle]. Prefer the family-specific name in new code.
pub use style::CartesianStyle as PlotStyle;
/// Compatibility name for [CartesianStyleProvider]. Prefer the family-specific name in new code.
pub use style::CartesianStyleProvider as PlotStyleProvider;

mod bar;
pub use bar::{BarPresentation, BarLabel, BarLabelPlacement};

pub(crate) mod pie;
pub use pie::{PiePresentation, PieLabelPlacement, PieStyleProvider, PieRingPresentation};

mod radar;
pub use radar::{RadarPresentation, RadarSeriesPresentation, RadarGridShape};

mod radial;
pub use radial::RadialPresentation;

mod annotation;
pub use annotation::AnnotationPresentation;
