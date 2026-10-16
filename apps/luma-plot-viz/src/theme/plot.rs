use gpui::{Hsla, px, rgb, linear_gradient, linear_color_stop};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_radix::{Look, SemanticRole};
use gpui_luma_plot::{
    AreaPresentation, AxisPresentation, ChartMetrics, EmptyStatePresentation, GridPresentation, LinePresentation,
    MarkerPresentation, CartesianStyle, CartesianStyleProvider, SeriesId, SeriesStyle, TextPresentation,
};

/// One live Radix adapter for all plot appearance. Series assignments are explicit
/// host choices; this adapter has no knowledge of dataset names or sample values.
#[derive(Clone)]
pub(crate) struct RadixCartesianStyle {
    look: Look,
    blue: bool,
    series: Vec<(SeriesId, usize)>,
}
impl RadixCartesianStyle {
    pub(crate) fn new(look: &Look) -> Self {
        Self { look: look.clone(), blue: false, series: Vec::new() }
    }
    /// Screenshot-style blue palette, resolved for the current light/dark mode.
    pub(crate) fn blue(mut self) -> Self {
        self.blue = true;
        self
    }
    pub(crate) fn series(mut self, id: impl Into<SeriesId>, color: usize) -> Self {
        self.series.push((id.into(), color));
        self
    }

    fn colors(&self) -> [Hsla; 2] {
        if self.blue {
            match self.look.mode() {
                ThemeMode::Dark => [rgb(0x93c5fd).into(), rgb(0x3b82f6).into()],
                ThemeMode::Light => [rgb(0x0284c7).into(), rgb(0x2563eb).into()],
            }
        } else {
            [
                self.look.resolve_role(SemanticRole::Primary).hsla(),
                self.look.resolve_role(SemanticRole::Focus).hsla(),
            ]
        }
    }
    fn series_style(&self, color: Hsla) -> SeriesStyle {
        let mut line = LinePresentation::new(color);
        line.markers = Some(MarkerPresentation::new(color, px(4.0)));
        let area = AreaPresentation::new(linear_gradient(
            180.0,
            linear_color_stop(color.opacity(if self.blue { 0.45 } else { 0.55 }), 0.0),
            linear_color_stop(color.opacity(0.02), 1.0),
        ))
        .outline(LinePresentation::new(color));
        SeriesStyle { line: Some(line), area: Some(area), bar: Some(gpui_luma_plot::BarPresentation::new(color)) }
    }
}
impl CartesianStyleProvider for RadixCartesianStyle {
    fn resolve(&self) -> CartesianStyle {
        let spacing = self.look.metrics().spacing;
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        let typography = crate::theme::typography();
        let caption = typography.text.caption;
        let text = TextPresentation {
            color: muted,
            font_family: typography.font.sans.family.into(),
            font_size: px(caption.size),
            line_height: px(caption.line_height),
        };
        let mut axis = AxisPresentation::new(muted).tick_labels(text.clone());
        axis.title = text;
        axis.gutter = px(spacing.s6 + spacing.s2);
        axis.tick_length = px(spacing.s1);
        axis.label_gap = px(spacing.s1);
        let colors = self.colors();
        let defaults = self.series_style(colors[0]);
        let mut style = CartesianStyle::new(axis, defaults.line.expect("adapter supplies a line"));
        style.area = defaults.area.expect("adapter supplies an area");
        style.grid = GridPresentation::new(self.look.resolve_role(SemanticRole::Border).hsla());
        style.metrics = ChartMetrics {
            plot_padding: px(spacing.s1),
            legend_gap: px(spacing.s3),
            legend_item_gap: px(spacing.s1),
            legend_swatch_width: px(spacing.s5),
            legend_swatch_height: px(spacing.s0.max(2.0)),
        };
        style.empty_state = Some(EmptyStatePresentation::new("No finite samples selected", muted));
        for (id, color) in &self.series {
            style.series.insert(id.clone(), self.series_style(colors[color % colors.len()]));
        }
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_radix_provider_resolves_current_mode_and_explicit_series_assignments() {
        let look = Look::built_in();
        look.set_mode(ThemeMode::Light);
        let provider = RadixCartesianStyle::new(&look).blue().series("desktop", 0).series("mobile", 1);
        let light = provider.resolve();
        look.set_mode(ThemeMode::Dark);
        let dark = provider.resolve();
        assert_ne!(light.axes[0].presentation.stroke.color, dark.axes[0].presentation.stroke.color);
        assert_ne!(
            light.series[&"desktop".into()].line.unwrap().stroke.color,
            dark.series[&"desktop".into()].line.unwrap().stroke.color
        );
        assert_ne!(
            dark.series[&"desktop".into()].line.unwrap().stroke.color,
            dark.series[&"mobile".into()].line.unwrap().stroke.color
        );
        assert!(dark.series[&"desktop".into()].area.unwrap().outline.is_some());
        assert_eq!(dark.series.len(), 2);
        assert_eq!(RadixCartesianStyle::new(&look).resolve().series.len(), 0);
    }
}
