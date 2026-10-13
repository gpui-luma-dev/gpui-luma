//! Public GPUI elements combining plot specifications and presentation.

mod builder;
mod cartesian;
mod diagnostics;
mod legend;

pub use builder::CartesianChartBuilder;
pub use cartesian::CartesianChartView;
pub use diagnostics::ChartRenderStats;
pub(crate) use diagnostics::RenderObserver;
pub use legend::{LegendItem, LegendView, LegendEvent, LegendInteraction, LegendTarget};

mod points;
pub use points::{PlotPoint, PlotSlice, PlotPointEvent};
pub(crate) use points::{PointTooltip, PointTooltipFactory, PointEvents};

mod style;
pub(crate) use style::StyleSource;

mod bar;
pub use bar::{BarRenderModel, BarTemplate, RectangleBarTemplate};
pub(crate) use bar::{BarTemplateSource, BarLabelSource, BarViewOptions};

pub(crate) mod pie;
pub use pie::{PieChartBuilder, PieChartView, PieSlice, PieSliceEvent};

pub(crate) mod radar;
pub use radar::{RadarChartBuilder, RadarChartView, RadarSpoke, RadarPoint};

pub(crate) mod radial;
pub use radial::{RadialChartBuilder, RadialChartView, RadialBar};

pub(crate) mod selection;
pub use selection::SelectionEvent;

mod annotation;
pub use annotation::ChartAnnotation;
