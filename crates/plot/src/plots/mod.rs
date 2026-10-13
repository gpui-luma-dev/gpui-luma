//! Plot specifications and data mappings, independent of styling and rendering.

mod cartesian;
mod line;
mod scale;

pub use cartesian::CartesianPlot;
pub use scale::{CategoryScale, NumericScale, Scale};
pub use line::{Decimation, Interpolation, LineLayer};

mod area;
pub use area::AreaLayer;

mod bar;
pub use bar::{BarLayer, BarLayout, BarOrientation, BarGrouping, BarAlignment};

mod pie;
pub use pie::{PiePlot};

mod radar;
pub use radar::RadarPlot;

mod radial;
pub use radial::{RadialPlot, RadialLayout};

mod annotation;
pub use annotation::Annotation;
