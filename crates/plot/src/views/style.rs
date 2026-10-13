use std::{fmt, sync::Arc};
use crate::{PlotStyle, PlotStyleProvider};

type Override = dyn Fn(&mut PlotStyle) + Send + Sync;

#[derive(Clone)]
pub(crate) struct StyleSource {
    provider: Arc<dyn PlotStyleProvider>,
    overrides: Vec<Arc<Override>>,
}
impl StyleSource {
    pub(crate) fn new(provider: impl PlotStyleProvider) -> Self {
        Self { provider: Arc::new(provider), overrides: Vec::new() }
    }
    pub(crate) fn customize(&mut self, customize: impl Fn(&mut PlotStyle) + Send + Sync + 'static) {
        self.overrides.push(Arc::new(customize));
    }
    pub(crate) fn resolve(&self) -> PlotStyle {
        let mut style = self.provider.resolve();
        for customize in &self.overrides {
            customize(&mut style);
        }
        style
    }
}
impl Default for StyleSource {
    fn default() -> Self {
        Self::new(PlotStyle::default())
    }
}
impl fmt::Debug for StyleSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StyleSource")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use crate::{
        AreaPresentation, CartesianChartBuilder, CartesianPresentation, Dataset, LineLayer, LinePresentation, Sample,
        Series,
    };

    #[test]
    fn live_resolution_reapplies_overrides_without_freezing_theme_or_mutating_provider() {
        let dark = Arc::new(AtomicBool::new(false));
        let current = dark.clone();
        let mut source = StyleSource::new(move || {
            let mut style = PlotStyle::default();
            style.line.stroke.color = if current.load(Ordering::Relaxed) {
                gpui::white()
            } else {
                gpui::black()
            };
            style.line.stroke.width = gpui::px(2.0);
            style
        });
        source.customize(|style| style.line.stroke.width = gpui::px(4.0));
        source.customize(|style| style.line.stroke.width += gpui::px(1.0));
        let first = source.resolve();
        dark.store(true, Ordering::Relaxed);
        let second = source.resolve();
        assert_eq!(first.line.stroke.color, gpui::black());
        assert_eq!(second.line.stroke.color, gpui::white());
        assert_eq!(first.line.stroke.width, gpui::px(5.0));
        assert_eq!(second.line.stroke.width, gpui::px(5.0));
        assert_eq!(source.provider.resolve().line.stroke.width, gpui::px(2.0));
    }

    #[test]
    fn series_overrides_merge_components_and_do_not_add_layers_or_change_samples() {
        let data = Dataset::new().with_series(Series::new("a", "A", [Sample::new(1.0, 2.0)])).expect("dataset");
        let samples = data.series()[0].samples().as_ptr();
        let mut style = PlotStyle::default();
        style.set_series_style("a".into(), LinePresentation::new(gpui::white()).into());
        style.set_series_style("a".into(), AreaPresentation::new(gpui::black()).into());
        let resolved = CartesianPresentation::from(style.clone());
        assert_eq!(resolved.line_for(&"a".into()).stroke.color, gpui::white());
        assert_eq!(resolved.area_for(&"a".into()).fill, gpui::Background::from(gpui::black()));
        assert!(resolved.area_for(&"a".into()).outline.is_none());
        let builder = CartesianChartBuilder::new(data.clone())
            .style(style)
            .series_style("unselected", LinePresentation::new(gpui::black()))
            .line(LineLayer::new("a"));
        let _ = builder.build();
        assert_eq!(data.series()[0].samples().as_ptr(), samples);
        assert_eq!(data.series().len(), 1);
    }
    struct RenderStyleHarness {
        chart: Option<crate::CartesianChartView>,
    }
    impl gpui::Render for RenderStyleHarness {
        fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
            use gpui::prelude::*;
            gpui::div().size_full().when_some(self.chart.take(), |root, chart| root.child(chart))
        }
    }

    #[gpui::test]
    fn chart_resolves_theme_at_render_time_instead_of_build_time(cx: &mut gpui::TestAppContext) {
        let dark = Arc::new(AtomicBool::new(false));
        let current = dark.clone();
        let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = observed.clone();
        let chart = CartesianChartBuilder::new(Dataset::new())
            .style(move || {
                let mut style = PlotStyle::default();
                style.line.stroke.color = if current.load(Ordering::Relaxed) {
                    gpui::white()
                } else {
                    gpui::black()
                };
                captured.lock().expect("observations").push(style.line.stroke.color);
                style
            })
            .build();
        assert!(observed.lock().expect("observations").is_empty());
        dark.store(true, Ordering::Relaxed);
        let _ = cx.add_window_view(|_, _| RenderStyleHarness { chart: Some(chart) });
        let observed = observed.lock().expect("observations");
        assert!(!observed.is_empty());
        assert!(observed.iter().all(|color| *color == gpui::white()));
    }
}
