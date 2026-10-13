//! Private transforms, geometry, and painting; backend types stay internal.

mod axis;
mod cartesian;
mod decimation;
mod grid;
mod line;
mod interpolation;
mod geometry;
mod viewport;

pub(crate) use cartesian::cartesian_canvas;

mod area;

mod points;

mod bar;

pub(crate) mod pie;

pub(crate) mod radar;

pub(crate) mod radial;

mod selection;

mod annotation;
