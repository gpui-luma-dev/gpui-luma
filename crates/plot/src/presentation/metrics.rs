use gpui::{Pixels, px};

/// Geometry resolved by a presentation adapter for a Cartesian plot.
#[derive(Clone, Copy, Debug)]
pub struct ChartMetrics {
    pub plot_padding: Pixels,
    pub legend_gap: Pixels,
    pub legend_item_gap: Pixels,
    pub legend_swatch_width: Pixels,
    /// Filled legend swatch height; line swatches use their stroke width.
    pub legend_swatch_height: Pixels,
}

impl Default for ChartMetrics {
    fn default() -> Self {
        Self {
            plot_padding: px(4.),
            legend_gap: px(8.),
            legend_item_gap: px(4.),
            legend_swatch_width: px(16.),
            legend_swatch_height: px(2.),
        }
    }
}
