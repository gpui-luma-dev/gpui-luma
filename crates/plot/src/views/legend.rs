use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};

use crate::presentation::{ChartMetrics, LegendPresentation, SwatchPresentation};

/// One resolved legend entry. Its label comes from data; its sample comes from presentation.
#[derive(Clone, Debug)]
pub struct LegendItem {
    pub label: SharedString,
    pub swatch: SwatchPresentation,
}

/// Legend activation; the host decides whether to select, highlight or hide content.
#[derive(Clone, Debug)]
pub enum LegendTarget {
    Series(crate::SeriesId),
    Sample { series: crate::SeriesId, sample_index: usize },
}
/// Activation identifies both the displayed entry and optional stable source identity.
#[derive(Clone, Debug)]
pub struct LegendEvent {
    pub target: Option<LegendTarget>,
    pub index: usize,
    pub item: LegendItem,
}
type LegendCallback = dyn Fn(LegendEvent, &mut App);
/// Host-supplied legend state; indices refer to the displayed item order.
#[derive(Clone, Default)]
pub struct LegendInteraction {
    pub selected: Vec<usize>,
    pub hidden: Vec<usize>,
    callback: Option<std::rc::Rc<LegendCallback>>,
}
impl std::fmt::Debug for LegendInteraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegendInteraction")
            .field("selected", &self.selected)
            .field("hidden", &self.hidden)
            .finish_non_exhaustive()
    }
}
impl LegendInteraction {
    pub fn selected(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.selected = indices.into_iter().collect();
        self
    }
    pub fn hidden(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.hidden = indices.into_iter().collect();
        self
    }
    pub fn on_activate(mut self, callback: impl Fn(LegendEvent, &mut App) + 'static) -> Self {
        self.callback = Some(std::rc::Rc::new(callback));
        self
    }
}
/// Stateless legend component that can also be placed outside the chart.
#[derive(IntoElement)]
pub struct LegendView {
    targets: Vec<LegendTarget>,
    items: Vec<LegendItem>,
    presentation: LegendPresentation,
    metrics: ChartMetrics,
    interaction: LegendInteraction,
}

impl LegendView {
    pub fn new(
        items: impl IntoIterator<Item = LegendItem>,
        presentation: LegendPresentation,
        metrics: ChartMetrics,
    ) -> Self {
        Self {
            targets: Vec::new(),
            items: items.into_iter().collect(),
            presentation,
            metrics,
            interaction: Default::default(),
        }
    }
    pub fn targets(mut self, targets: impl IntoIterator<Item = LegendTarget>) -> Self {
        self.targets = targets.into_iter().collect();
        self
    }
    pub fn interaction(mut self, interaction: LegendInteraction) -> Self {
        self.interaction = interaction;
        self
    }
}

impl RenderOnce for LegendView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let metrics = self.metrics;
        div()
            .flex()
            .flex_wrap()
            .flex_none()
            .gap(metrics.legend_gap)
            .when(self.presentation.visible, |this| {
                this.children(self.items.into_iter().enumerate().map(|(index, item)| {
                    let callback = self.interaction.callback.clone();
                    let event = LegendEvent { target: self.targets.get(index).cloned(), index, item: item.clone() };
                    let (paint, height) = match item.swatch {
                        SwatchPresentation::Line(stroke) => {
                            let height = if f32::from(stroke.width).is_finite() && stroke.width > gpui::px(0.0) {
                                stroke.width
                            } else {
                                gpui::px(0.0)
                            };
                            (stroke.color.into(), height)
                        }
                        SwatchPresentation::Fill(paint) => (paint, metrics.legend_swatch_height),
                    };
                    div()
                        .id(("legend-item", index))
                        .opacity(if self.interaction.hidden.contains(&index) {
                            0.4
                        } else {
                            1.0
                        })
                        .when(self.interaction.selected.contains(&index), |item| {
                            item.border_b_2().border_color(self.presentation.text_color)
                        })
                        .when_some(callback, |item, callback| {
                            item.cursor_pointer().on_click(move |_, _, cx| callback(event.clone(), cx))
                        })
                        .flex()
                        .items_center()
                        .gap(metrics.legend_item_gap)
                        .child(div().w(metrics.legend_swatch_width).h(height).flex_shrink_0().bg(paint))
                        .child(div().text_color(self.presentation.text_color).child(item.label))
                }))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, px, point};
    struct Harness {
        events: std::rc::Rc<std::cell::RefCell<Vec<LegendEvent>>>,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let events = self.events.clone();
            div().w(px(300.0)).h(px(40.0)).child(
                LegendView::new(
                    [LegendItem {
                        label: "Desktop".into(),
                        swatch: SwatchPresentation::Fill(gpui::rgb(0x93c5fd).into()),
                    }],
                    LegendPresentation::new(gpui::black()),
                    ChartMetrics::default(),
                )
                .interaction(
                    LegendInteraction::default()
                        .selected([0])
                        .hidden([0])
                        .on_activate(move |event, _| events.borrow_mut().push(event)),
                ),
            )
        }
    }
    #[gpui::test]
    fn styled_legend_activation_reports_displayed_item_without_owning_selection(cx: &mut TestAppContext) {
        let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|_, _| Harness { events: events.clone() });
        cx.simulate_click(point(px(12.0), px(10.0)), gpui::Modifiers::default());
        assert_eq!(events.borrow().len(), 1);
        assert_eq!(events.borrow()[0].index, 0);
        assert_eq!(events.borrow()[0].item.label, "Desktop");
    }
}
