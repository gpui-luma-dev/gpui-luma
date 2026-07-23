use std::sync::Arc;

use gpui::{Context, Entity, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::command::button::HasPresenter;
use gpui_luma::controls::control_group::ControlGroupEvent;
use gpui_luma::controls::list_view::ScrollingListView;
use gpui_luma::controls::listbox::ListBox;
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::controls::toggle::{Toggle, ToggleEvent};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::activity::{RideActivity, compute_activity_power_curve};
use super::content_tabs::graph_viz_tabs_navigation_template;
use super::cycling_dynamics_tab::render_cycling_dynamics_tab;
use super::graph_tab::{
    default_selected_metric_ids, metric_listbox_items, power_unit_toggle_label, render_graph_tab,
    visible_metrics_from_ids,
};
use super::laps_tab::{refresh_laps_list_view, render_laps_tab, spawn_laps_list_view, LapRow};
use super::metrics::TelemetryMetric;
use super::power_curve_tab::render_power_curve_tab;
use super::stats_tab::render_stats_tab;
use super::units::SpeedUnit;
use super::zones_tab::render_zones_tab;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContentTab {
    #[default]
    Graph,
    Stats,
    Laps,
    TimeInZones,
    CyclingDynamics,
    PowerCurve,
}

impl ContentTab {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            "graph" => Some(Self::Graph),
            "stats" => Some(Self::Stats),
            "laps" => Some(Self::Laps),
            "zones" => Some(Self::TimeInZones),
            "cycling-dynamics" => Some(Self::CyclingDynamics),
            "power-curve" => Some(Self::PowerCurve),
            _ => None,
        }
    }
}

pub struct ContentPaneHost {
    tabs: Entity<TabsNavigation>,
    look: Arc<ShadcnLook>,
    ride: RideActivity,
    available_metrics: Vec<TelemetryMetric>,
    metric_listbox: ListBox,
    speed_unit_toggle: Toggle,
    power_unit_toggle: Toggle,
    x_axis_toggle: Toggle,
    timeline_slider: Slider,
    laps_list_view: ScrollingListView<LapRow>,
    scrub_fraction: f32,
    active_tab: ContentTab,
    _subscriptions: Vec<Subscription>,
}

impl ContentPaneHost {
    pub fn new(look: Arc<ShadcnLook>, ride: RideActivity, cx: &mut Context<Self>) -> Self {
        let available_metrics = TelemetryMetric::available(&ride.points);
        let metric_listbox = look
            .listbox_multiple("graph-viz-metrics")
            .items(metric_listbox_items(&available_metrics))
            .selected_ids(default_selected_metric_ids(&available_metrics))
            .spawn(cx);
        let speed_unit_toggle = look
            .secondary_toggle("graph-viz-speed-unit")
            .with_data(false)
            .content(|model, _| speed_unit_toggle_label(model.data).into_any_element())
            .spawn(cx);
        let power_unit_toggle = look
            .secondary_toggle("graph-viz-power-unit")
            .with_data(false)
            .content(|model, _| power_unit_toggle_label(model.data).into_any_element())
            .spawn(cx);
        let x_axis_toggle = look
            .secondary_toggle("graph-viz-x-axis")
            .with_data(false)
            .content(|model, _| x_axis_toggle_label(model.data).into_any_element())
            .spawn(cx);
        let timeline_slider = look.slider("graph-viz-timeline").range(0..100).step(1).value(0.0).spawn(cx);
        let laps_list_view = spawn_laps_list_view(&look, &ride.laps, SpeedUnit::Mph, cx);

        let tabs = look
            .tabs_navigation("graph-viz-content-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(graph_viz_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("graph").label("Graph"),
                TabsNavigationItem::new("stats").label("Stats"),
                TabsNavigationItem::new("laps").label("Laps"),
                TabsNavigationItem::new("zones").label("Time In Zones"),
                TabsNavigationItem::new("cycling-dynamics").label("Cycling Dynamics"),
                TabsNavigationItem::new("power-curve").label("Power Curve"),
            ])
            .active("graph")
            .spawn(cx);

        let tabs_for_sub = tabs.clone();
        let metric_listbox_for_sub = metric_listbox.clone();
        let speed_unit_toggle_for_sub = speed_unit_toggle.clone();
        let power_unit_toggle_for_sub = power_unit_toggle.clone();
        let x_axis_toggle_for_sub = x_axis_toggle.clone();
        let timeline_slider_for_sub = timeline_slider.clone();
        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs_for_sub, |host, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event else {
                return;
            };
            if let Some(tab) = ContentTab::from_id(tab_id.as_ref())
                && host.active_tab != tab
            {
                host.active_tab = tab;
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&metric_listbox_for_sub, |_, _, _: &ControlGroupEvent, cx| {
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&speed_unit_toggle_for_sub, |host, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected } = event {
                let speed_unit = SpeedUnit::from_toggle(*selected);
                refresh_laps_list_view(&host.laps_list_view, &host.ride.laps, speed_unit, cx);
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&power_unit_toggle_for_sub, |_, _, event: &ToggleEvent, cx| {
            if matches!(event, ToggleEvent::Change { .. }) {
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&x_axis_toggle_for_sub, |_, _, event: &ToggleEvent, cx| {
            if matches!(event, ToggleEvent::Change { .. }) {
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe(&timeline_slider_for_sub, |host, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } = event {
                host.scrub_fraction = (*value / 100.0).clamp(0.0, 1.0);
                cx.notify();
            }
        }));

        Self {
            tabs,
            look,
            ride,
            available_metrics,
            metric_listbox,
            speed_unit_toggle,
            power_unit_toggle,
            x_axis_toggle,
            timeline_slider,
            laps_list_view,
            scrub_fraction: 0.0,
            active_tab: ContentTab::Graph,
            _subscriptions: subscriptions,
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsNavigationWidthMode::Intrinsic, cx);
            tabs.set_template(graph_viz_tabs_navigation_template(look.clone(), ControlSize::Lg), cx);
        });
        self.metric_listbox.update(cx, |listbox, cx| {
            listbox.set_template(look.listbox_template(), cx);
        });
        self.speed_unit_toggle.update(cx, |toggle, cx| {
            toggle.set_template(look.toggle_template(gpui_luma_look_shadcn::ShadcnButtonStyle::Secondary), cx);
        });
        self.power_unit_toggle.update(cx, |toggle, cx| {
            toggle.set_template(look.toggle_template(gpui_luma_look_shadcn::ShadcnButtonStyle::Secondary), cx);
        });
        self.x_axis_toggle.update(cx, |toggle, cx| {
            toggle.set_template(look.toggle_template(gpui_luma_look_shadcn::ShadcnButtonStyle::Secondary), cx);
        });
        self.timeline_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.laps_list_view.update(cx, |list, cx| {
            list.set_template(look.list_view_template(), cx);
            list.set_theme(look.list_view_theme(), cx);
        });
        cx.notify();
    }

    pub fn notify_tabs(&self, cx: &mut Context<Self>) {
        self.tabs.update(cx, |_, cx| cx.notify());
    }

    fn visible_metrics(&self, cx: &Context<Self>) -> Vec<TelemetryMetric> {
        let selected_ids: Vec<SharedString> =
            self.metric_listbox.read(cx).selected_ids().iter().map(|id| (*id).clone()).collect();
        visible_metrics_from_ids(&self.available_metrics, &selected_ids)
    }
}

fn speed_unit_toggle_label(kmh_selected: bool) -> SharedString {
    SharedString::from(if kmh_selected { "km/h" } else { "mph" })
}

fn x_axis_toggle_label(distance_selected: bool) -> SharedString {
    SharedString::from(if distance_selected { "Distance" } else { "Time" })
}

impl Render for ContentPaneHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let chrome = self.look.chrome();
        let board_bg = self.look.token_color("background").unwrap_or(chrome.app_background);
        let visible_metrics = self.visible_metrics(cx);
        let speed_unit = SpeedUnit::from_toggle(*self.speed_unit_toggle.read(cx).data());

        div()
            .id("graph-viz-content-pane")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(board_bg)
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(match self.active_tab {
                ContentTab::Graph => div()
                    .id("graph-viz-graph-tab")
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .overflow_y_scroll()
                    .p(px(24.0))
                    .child(render_graph_tab(
                        &self.ride,
                        &visible_metrics,
                        self.scrub_fraction,
                        self.metric_listbox.clone(),
                        self.speed_unit_toggle.clone(),
                        self.power_unit_toggle.clone(),
                        self.x_axis_toggle.clone(),
                        self.timeline_slider.clone(),
                        &self.look,
                        window,
                        cx,
                    ))
                    .into_any_element(),
                ContentTab::Stats => div()
                    .id("graph-viz-stats-tab")
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .overflow_y_scroll()
                    .p(px(24.0))
                    .child(render_stats_tab(&self.ride, speed_unit, &self.look, window, cx))
                    .into_any_element(),
                ContentTab::Laps => div()
                    .id("graph-viz-laps-tab")
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .overflow_y_scroll()
                    .p(px(24.0))
                    .child(render_laps_tab(&self.ride, &self.laps_list_view, &self.look, window, cx))
                    .into_any_element(),
                ContentTab::TimeInZones => {
                    let profile = self.ride.zones.as_ref().cloned().unwrap_or_default();
                    div()
                        .id("graph-viz-zones-tab")
                        .flex_1()
                        .min_h_0()
                        .size_full()
                        .overflow_y_scroll()
                        .p(px(24.0))
                        .child(render_zones_tab(&profile, &self.look, window, cx))
                        .into_any_element()
                }
                ContentTab::CyclingDynamics => div()
                    .id("graph-viz-cycling-dynamics-tab")
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .overflow_y_scroll()
                    .p(px(24.0))
                    .child(render_cycling_dynamics_tab(&self.ride, &self.look, window, cx))
                    .into_any_element(),
                ContentTab::PowerCurve => {
                    let curve = compute_activity_power_curve(&self.ride.points);
                    div()
                        .id("graph-viz-power-curve-tab")
                        .flex_1()
                        .min_h_0()
                        .size_full()
                        .overflow_y_scroll()
                        .p(px(24.0))
                        .child(render_power_curve_tab(&curve, &self.look, window, cx))
                        .into_any_element()
                }
            })
    }
}
