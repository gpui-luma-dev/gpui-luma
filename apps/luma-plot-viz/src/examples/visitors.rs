use gpui_luma::motion::{Transition, Easing};
mod data;

use gpui_luma::controls::selector::SelectorEventExt;
use std::time::Duration;

use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::button::ButtonEventExt;
use gpui_luma::{
    controls::button::{Button},
    motion::VisualTransition,
    controls::selector::{Selector, SelectorEvent},
    hstack, vstack,
};
use gpui_luma_look_radix::Look;
use gpui_luma_plot::{
    AreaLayer, LineLayer, AxisSide, CartesianChartBuilder, ChartMetrics, Dataset, DatasetError, Interpolation,
    LegendItem, LegendPresentation, LegendView, SwatchPresentation, NumericScale, CartesianStyleProvider,
};
use crate::theme::{self, TypographyExt};

pub(crate) struct VisitorsExample {
    entrance: f32,
    data: Dataset,
    look: Look,
    area: bool,
    days: usize,
    range: Entity<Selector>,
    replay: Entity<Button<()>>,
    motion: VisualTransition,
    _subscriptions: Vec<Subscription>,
}

impl VisitorsExample {
    pub(crate) fn spawn(look: &Look, area: bool, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = data::dataset()?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let range = theme::visitors::spawn_range(&look, cx);
            range.clone().on_event(cx, |this, event: &SelectorEvent, cx| {
                if let SelectorEvent::Change { item_id, .. } = event {
                    this.days = match item_id.as_ref() {
                        "7" => 7,
                        "30" => 30,
                        _ => 91,
                    };
                    this.restart_motion();
                    cx.notify();
                }
            });
            let replay = theme::spawn_replay("visitors-replay", &look, cx).on_click(cx, |this, cx| {
                this.restart_motion();
                cx.notify();
            });
            Self {
                entrance: 1.0,
                data,
                look,
                area,
                days: 91,
                range,
                replay,
                motion: VisualTransition::with_transition(
                    1.0,
                    Transition::timed(Duration::from_millis(1400), Easing::OutCubic),
                ),
                _subscriptions: { Vec::new() },
            }
        }))
    }

    fn restart_motion(&mut self) {
        self.motion.snap_to(0.0);
        self.motion.set_target(1.0);
    }
}

impl Render for VisitorsExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        let style = theme::RadixCartesianStyle::new(&self.look)
            .blue()
            .series("desktop", 0)
            .series("mobile", 1)
            .resolve();
        let caption = if self.days == 91 {
            "Showing total visitors for the last 3 months".to_owned()
        } else {
            format!("Showing total visitors for the last {} days", self.days)
        };
        let metrics = ChartMetrics {
            legend_swatch_width: px(8.0),
            legend_swatch_height: px(8.0),
            legend_gap: px(20.0),
            legend_item_gap: px(8.0),
            ..ChartMetrics::default()
        };
        let legend = LegendView::new(
            [("desktop", "Desktop"), ("mobile", "Mobile")].into_iter().filter_map(|(id, label)| {
                let line = style.series.get(&id.into())?.line?;
                Some(LegendItem { label: label.into(), swatch: SwatchPresentation::Fill(line.stroke.color.into()) })
            }),
            LegendPresentation::new(chrome.body_text),
            metrics,
        );
        theme::panel_frame(&self.look, "plot-viz-visitors-frame")
            .render(cx)
            .child(
                vstack! { gap=0;
                    hstack! { gap=16;
                        vstack! { gap=4;
                            div().typography_style(typography.text.scale.lg).font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(if self.area { "Area Chart – Interactive" } else { "Line Chart – Interactive" }),
                            div().typography_style(typography.text.body).text_color(chrome.muted_text).child(caption),
                        }.flex_1().min_w_0(),
                        hstack! { self.replay.clone() }.flex_shrink_0(),
                        hstack! { self.range.clone() }.w(px(180.0)).flex_shrink_0(),
                    }.items_center().p_6().border_b_1().border_color(chrome.border),
                    vstack! { gap=16;
                        vstack! { chart(self.data.clone(), &self.look, self.days, self.area).draw_progress(self.motion.progress() * self.entrance).build() }.h(px(270.0)),
                        hstack! { justify=center; legend }.w_full(),
                    }.p_6(),
                }
                .w_full(),
            )
            .w_full()
    }
}

fn chart(data: Dataset, look: &Look, days: usize, area: bool) -> CartesianChartBuilder {
    let mut chart = CartesianChartBuilder::new(data)
        .style(theme::RadixCartesianStyle::new(look).blue().series("desktop", 0).series("mobile", 1))
        .x_scale(
            NumericScale::new()
                .domain(data::first_day(days) as f64..=data::LAST_DAY as f64)
                .tick_labels(data::ticks(days)),
        )
        .y_scale(NumericScale::new().domain(0.0..=700.0).tick_count(5))
        .style_override(|style| {
            style.axes.retain(|axis| axis.side == AxisSide::Bottom);
            for axis in &mut style.axes {
                axis.presentation.stroke.width = px(0.0);
                axis.presentation.tick_length = px(0.0);
                axis.presentation.gutter = px(28.0);
            }
            style.grid.vertical = None;
            style.legend.visible = false;
        });
    for id in ["desktop", "mobile"] {
        chart = if area {
            chart.area(AreaLayer::new(id).interpolation(Interpolation::Monotone))
        } else {
            chart.line(LineLayer::new(id).interpolation(Interpolation::Monotone))
        };
    }
    chart
}

crate::examples::motion::entrance!(VisitorsExample);
