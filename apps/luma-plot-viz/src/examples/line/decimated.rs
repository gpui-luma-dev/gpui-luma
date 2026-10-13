use gpui::px;
use gpui_luma_look_radix::Look;
use gpui_luma_plot::{CartesianChartBuilder, Dataset, Decimation};

use super::{LineExample, dense};
use crate::theme;

pub(super) fn example(data: Dataset, look: &Look) -> LineExample {
    LineExample {
        title: "Decimated time series · same 20,000 samples",
        description: "The same data and axes as the raw plot above. Min/max decimation retains endpoints and peaks per pixel column; missing intervals stay disconnected. Resize the window to change drawing resolution.",
        frame: theme::panel_frame(look, "plot-viz-decimated-frame"),
        height: px(360.0),
        data,
        chart,
        diagnostics: None,
    }
}

fn chart(data: Dataset, look: &Look) -> CartesianChartBuilder {
    dense::chart_with_decimation(data, look, Decimation::MinMax)
}
