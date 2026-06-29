use gpui::Hsla;
use gpui_luma_look_shadcn::ShadcnLook;

use super::metrics::TelemetryMetric;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartTheme {
    pub background: Hsla,
    pub grid_line: Hsla,
    pub axis_label: Hsla,
    pub cursor_line: Hsla,
    pub elevation_stroke: Hsla,
    pub heartrate_stroke: Hsla,
    pub respiration_stroke: Hsla,
    pub pco_stroke: Hsla,
    pub left_pco_stroke: Hsla,
    pub power_stroke: Hsla,
    pub speed_stroke: Hsla,
    pub speed_fill: Hsla,
}

impl ChartTheme {
    pub fn stroke_for_metric(self, metric: TelemetryMetric) -> Hsla {
        match metric {
            TelemetryMetric::Elevation => self.elevation_stroke,
            TelemetryMetric::HeartRate => self.heartrate_stroke,
            TelemetryMetric::RespirationRate => self.respiration_stroke,
            TelemetryMetric::RightPlatformCenterOffset => self.pco_stroke,
            TelemetryMetric::Power => self.power_stroke,
            TelemetryMetric::Speed => self.speed_stroke,
        }
    }

    pub fn fill_for_metric(self, metric: TelemetryMetric) -> Option<Hsla> {
        match metric {
            TelemetryMetric::Speed => Some(self.speed_fill),
            _ => None,
        }
    }
}

pub trait GraphVizThemeExt {
    fn resolve_chart_theme(&self) -> ChartTheme;
}

impl GraphVizThemeExt for ShadcnLook {
    fn resolve_chart_theme(&self) -> ChartTheme {
        let chrome = self.chrome();
        let speed_stroke = self.token_color("primary").unwrap_or(chrome.title_text);
        let speed_fill = self.token_color("chart-1").unwrap_or(speed_stroke).opacity(0.42);
        ChartTheme {
            background: chrome.panel_background,
            grid_line: self.token_color("border").unwrap_or(chrome.border),
            axis_label: chrome.muted_text,
            cursor_line: self.token_color("chart-2").unwrap_or(chrome.muted_text),
            elevation_stroke: self.token_color("muted-foreground").unwrap_or(chrome.title_text),
            heartrate_stroke: self.token_color("destructive").unwrap_or(chrome.title_text),
            respiration_stroke: self.token_color("chart-3").unwrap_or(chrome.title_text),
            pco_stroke: self.token_color("muted-foreground").unwrap_or(chrome.muted_text).opacity(0.72),
            left_pco_stroke: self.token_color("destructive").unwrap_or(chrome.title_text).opacity(0.72),
            power_stroke: self.token_color("chart-4").unwrap_or(chrome.title_text),
            speed_stroke,
            speed_fill,
        }
    }
}
