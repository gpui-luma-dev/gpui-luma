use std::{fmt, sync::Arc};
use gpui::{AnyElement, App, Bounds, IntoElement, Pixels, Point, Window, div, prelude::*};
use crate::{BarOrientation, BarPresentation, Sample, SeriesId};

/// Screen placement of one bar, before plot clipping. Bounds use window coordinates.
/// The template's element is laid out at `bounds.origin` with `bounds.size` available.
/// Baseline and endpoint lie at the bar center across its thickness; stacked bars receive their
/// own segment start/end. Samples and identities remain those of the source data.
#[derive(Clone, Debug)]
pub struct BarRenderModel {
    pub bounds: Bounds<Pixels>,
    pub baseline: Point<Pixels>,
    pub endpoint: Point<Pixels>,
    pub orientation: BarOrientation,
    pub presentation: BarPresentation,
    pub series: SeriesId,
    pub sample: Sample,
    pub sample_index: usize,
}

/// Replace a bar's visual without implementing scales, grouping, or stacking.
/// Return a local GPUI element sized to the supplied bounds. All shapes are clipped
/// to the plot and draw-progress mask. Interactive bar picking is not provided.
pub trait BarTemplate: 'static {
    fn render(&self, model: &BarRenderModel, window: &mut Window, cx: &mut App) -> AnyElement;
}
impl<F, E> BarTemplate for F
where
    F: Fn(&BarRenderModel, &mut Window, &mut App) -> E + 'static,
    E: IntoElement,
{
    fn render(&self, model: &BarRenderModel, window: &mut Window, cx: &mut App) -> AnyElement {
        self(model, window, cx).into_any_element()
    }
}

/// Default rectangular template; equivalent to the renderer's fast default fill.
#[derive(Clone, Copy, Debug, Default)]
pub struct RectangleBarTemplate;
impl BarTemplate for RectangleBarTemplate {
    fn render(&self, model: &BarRenderModel, _: &mut Window, _: &mut App) -> AnyElement {
        div()
            .w(model.bounds.size.width)
            .h(model.bounds.size.height)
            .bg(model.presentation.fill)
            .into_any_element()
    }
}
#[derive(Clone)]
pub(crate) struct BarTemplateSource(pub Arc<dyn BarTemplate>);
impl fmt::Debug for BarTemplateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BarTemplate")
    }
}

type BarLabelFactory = dyn Fn(&BarRenderModel) -> Vec<crate::BarLabel>;
#[derive(Clone)]
pub(crate) struct BarLabelSource(pub Arc<BarLabelFactory>);
impl fmt::Debug for BarLabelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BarLabels")
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct BarViewOptions {
    pub template: Option<BarTemplateSource>,
    pub labels: Option<BarLabelSource>,
    pub tooltip: Option<super::PointTooltipFactory>,
    pub motion: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BarLayer, BarLayout, BarGrouping, CategoryScale, CartesianChartBuilder, Dataset, NumericScale, Series};
    use gpui::{Context, Render, TestAppContext, px};
    use std::{cell::RefCell, rc::Rc};

    struct Harness {
        models: Rc<RefCell<Vec<BarRenderModel>>>,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let models = self.models.clone();
            let data = Dataset::new()
                .with_series(Series::new("a", "A", [Sample::new(0.0, 3.0), Sample::new(1.0, f64::NAN)]))
                .unwrap()
                .with_series(Series::new("b", "B", [Sample::new(0.0, 2.0)]))
                .unwrap();
            div().w(px(300.0)).h(px(200.0)).child(
                CartesianChartBuilder::new(data)
                    .x_scale(CategoryScale::new(["A", "B"]))
                    .y_scale(NumericScale::new().domain(0.0..=5.0))
                    .bar_orientation(BarOrientation::Left)
                    .bar_layout(BarLayout::new().grouping(BarGrouping::Stacked))
                    .bar(BarLayer::new("a"))
                    .bar(BarLayer::new("b"))
                    .bar_value_labels(crate::TextPresentation::new(gpui::white()))
                    .bar_template(move |model: &BarRenderModel, _: &mut Window, _: &mut App| {
                        models.borrow_mut().push(model.clone());
                        div()
                            .w(model.bounds.size.width)
                            .h(model.bounds.size.height)
                            .rounded_md()
                            .bg(model.presentation.fill)
                    })
                    .build(),
            )
        }
    }
    #[gpui::test]
    fn custom_template_is_laid_out_and_painted_with_resolved_stack_segments(cx: &mut TestAppContext) {
        let models = Rc::new(RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|_, _| Harness { models: models.clone() });
        models.borrow_mut().clear();
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        let models = models.borrow();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].orientation, BarOrientation::Left);
        assert_eq!(models[0].series, "a".into());
        assert_eq!(models[1].sample, Sample::new(0.0, 2.0));
        assert_eq!(models[1].baseline, models[0].endpoint);
        assert!(models[1].endpoint.x < models[1].baseline.x);
        assert!(models.iter().all(|bar| bar.bounds.size.width > px(0.0) && bar.bounds.size.height > px(0.0)));
    }
}
