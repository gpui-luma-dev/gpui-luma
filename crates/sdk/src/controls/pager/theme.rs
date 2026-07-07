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
            button_size: if compact {
                28.0
            } else {
                metrics.control_height(crate::theme::ControlSize::Sm)
            },
            button_min_width: if compact { 28.0 } else { 32.0 },
            control_height: metrics.control_height(crate::theme::ControlSize::Sm),
            radius: metrics.radius(crate::theme::ControlSize::Sm),
            padding_x: 8.0,
            padding_y: 8.0,
            gap: 4.0,
            group_gap: if compact { 8.0 } else { 16.0 },
            disabled_opacity: 0.56,
        }
    }
}
