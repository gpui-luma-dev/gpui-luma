use std::sync::{Arc, OnceLock};

use gpui::{FontWeight, Hsla};

use crate::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyRole, compose_button_family_look, default_button_family_theme,
};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate, default_button_template};
use crate::theme::{ControlSize, LumaTextStyle, StandardBoxScale, ThemeTokens};

use super::PagerStyle;

#[derive(Clone, Debug)]
pub struct PagerLook {
    pub panel_background: Hsla,
    pub border: Hsla,
    pub body_text: Hsla,
    pub muted_text: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub hover_background: Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    pub typography: LumaTextStyle,
    pub button_size: f32,
    pub button_min_width: f32,
    pub control_height: f32,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub group_gap: f32,
    pub disabled_opacity: f32,
}

pub trait PagerTheme: Send + Sync {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook;

    fn button_template(&self) -> Arc<dyn ButtonTemplate<()>> {
        default_button_template()
    }

    fn resolve_button_look(&self, pager_look: &PagerLook, model: &ButtonRenderModel<()>) -> ButtonFamilyLook {
        let button_theme = default_button_family_theme();
        let palette = button_theme.resolve(model.role, ControlSize::Sm, model.state);
        let square = matches!(model.role, ButtonFamilyRole::Icon);
        let scale = StandardBoxScale {
            height: pager_look.button_size,
            padding_x: if square { 0.0 } else { pager_look.padding_x },
            padding_y: 0.0,
            gap: pager_look.gap,
            radius: pager_look.radius,
        };
        let base = compose_button_family_look(&palette, model.role, &scale, pager_look.radius);
        tune_pager_button_look(base, pager_look, model.role)
    }
}

pub fn tune_pager_button_look(
    mut look: ButtonFamilyLook,
    pager_look: &PagerLook,
    role: ButtonFamilyRole,
) -> ButtonFamilyLook {
    look.height = pager_look.button_size;
    look.radius = pager_look.radius;
    look.gap = pager_look.gap;
    look.typography.size = pager_look.typography.size;
    look.typography.line_height = pager_look.typography.line_height;
    look.typography.weight = if matches!(role, ButtonFamilyRole::Toggle { selected: true }) {
        FontWeight::SEMIBOLD
    } else {
        pager_look.typography.weight
    };
    if matches!(role, ButtonFamilyRole::Icon) {
        look.padding_x = 0.0;
        look.padding_y = 0.0;
        look.icon_size = pager_look.button_size * 0.44;
    } else {
        look.padding_x = pager_look.padding_x;
        look.padding_y = 0.0;
    }
    look
}

#[derive(Clone, Debug, Default)]
pub struct DefaultPagerTheme {
    tokens: ThemeTokens,
}

pub fn default_pager_theme() -> Arc<dyn PagerTheme> {
    static THEME: OnceLock<Arc<dyn PagerTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultPagerTheme::default())).clone()
}

impl DefaultPagerTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl PagerTheme for DefaultPagerTheme {
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = self.tokens.typography.text.caption;
        let compact = matches!(style, PagerStyle::Minimal);
        let control_size = ControlSize::Sm;
        let control_height = metrics.control_height(control_size);
        let compact_gap = metrics.spacing.s3;
        let regular_gap = metrics.spacing.s5;
        let item_gap = metrics.spacing.s1;
        let padding_x = (metrics.padding_x(control_size) - (metrics.spacing.s1 * 0.5)).max(0.0);
        let padding_y = metrics.padding_y(control_size) + (metrics.gap(control_size) * 0.5);

        PagerLook {
            panel_background: if enabled {
                palette.surface.panel.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            body_text: if enabled {
                palette.app.foreground
            } else {
                palette.state.disabled.foreground
            },
            muted_text: if enabled {
                palette.app.muted_foreground
            } else {
                palette.state.disabled.foreground
            },
            selected_background: palette.state.selected.background,
            selected_foreground: palette.state.selected.foreground,
            hover_background: palette.state.hover.background,
            shadow: vec![],
            typography,
            button_size: control_height,
            button_min_width: if compact {
                control_height
            } else {
                control_height + metrics.spacing.s1
            },
            control_height,
            radius: metrics.radius(control_size),
            padding_x,
            padding_y,
            gap: item_gap,
            group_gap: if compact { compact_gap } else { regular_gap },
            disabled_opacity: 0.56,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ControlMetricTokens, MetricTokens};

    #[test]
    fn default_pager_fallback_uses_metric_tokens() {
        let mut tokens = ThemeTokens::default();
        tokens.metrics = MetricTokens::default();
        tokens.metrics.control.sm = ControlMetricTokens::new(40.0, 18.0, 7.0, 10.0, 9.0);
        tokens.metrics.spacing.s1 = 5.0;
        tokens.metrics.spacing.s3 = 11.0;
        tokens.metrics.spacing.s5 = 19.0;

        let theme = DefaultPagerTheme::new(tokens);
        let regular = theme.resolve(true, PagerStyle::Numeric);
        let compact = theme.resolve(true, PagerStyle::Minimal);

        assert_eq!(regular.button_size, 40.0);
        assert_eq!(regular.button_min_width, 45.0);
        assert_eq!(regular.control_height, 40.0);
        assert_eq!(regular.radius, 9.0);
        assert_eq!(regular.padding_x, 15.5);
        assert_eq!(regular.padding_y, 12.0);
        assert_eq!(regular.gap, 5.0);
        assert_eq!(regular.group_gap, 19.0);
        assert_eq!(compact.button_min_width, 40.0);
        assert_eq!(compact.group_gap, 11.0);
    }
}
