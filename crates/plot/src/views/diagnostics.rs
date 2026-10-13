use std::{fmt, rc::Rc, time::Duration};

use gpui::App;

/// One measured chart canvas render. Counts include each selected line, area, or bar layer.
#[derive(Clone, Copy, Debug)]
pub struct ChartRenderStats {
    /// Stored observations, including missing or unmapped coordinates.
    pub source_points: usize,
    /// Accepted observations before decimation.
    pub valid_points: usize,
    /// Selected viewport observations before decimation, including edge neighbors.
    pub visible_points: usize,
    /// Visible X domain; automatic or explicitly selected.
    pub x_domain: (f64, f64),
    /// Drawing points after decimation (including singleton runs), or projected bar rectangles.
    pub retained_points: usize,
    pub runs: usize,
    /// Plot area in logical pixels, excluding axis gutters.
    pub plot_width: f32,
    /// Horizontal plot inset from the canvas origin, in logical pixels.
    pub plot_offset_x: f32,
    pub plot_height: f32,
    /// CPU time for scale scans, axis measurement, projection, and decimation.
    pub prepare_time: Duration,
    /// CPU time for grid, axis, and line path construction and paint submission.
    /// Excludes GPU execution, host layout, and the legend.
    pub paint_time: Duration,
}

type Observer = dyn Fn(ChartRenderStats, &mut App);

#[derive(Clone)]
pub(crate) struct RenderObserver(Rc<Observer>);

impl RenderObserver {
    pub(crate) fn new(callback: impl Fn(ChartRenderStats, &mut App) + 'static) -> Self {
        Self(Rc::new(callback))
    }

    pub(crate) fn report(&self, stats: ChartRenderStats, cx: &mut App) {
        (self.0)(stats, cx);
    }
}

impl fmt::Debug for RenderObserver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RenderObserver")
    }
}
