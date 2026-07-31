#![allow(clippy::too_many_arguments)]

mod units;

pub mod activity;

mod app;
mod chart_theme;
mod charts;
mod content_pane;
mod content_tabs;
mod controls;
mod graph_tab;
mod metrics;
mod plot;
mod laps_tab;
mod ride_summary;
mod stats_tab;
mod cycling_dynamics_tab;
mod power_curve_tab;
mod theme_sidebar;
mod zones_tab;

pub use app::GraphVizApp;
