use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::controls::button_family::{
    ButtonFamilyRole, ButtonFamilyTheme, DefaultButtonFamilyTheme, compose_button_family_look,
};
use crate::theme::{ControlSize, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale, ThemeTokens};
use crate::theme::adorner::AdornerSpec;

#[derive(Clone, Debug)]
pub struct ControlGroupItemVisualContext {
    pub foreground: Hsla,
    pub background: Hsla,
    pub muted_foreground: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub adorner: Option<AdornerSpec>,
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlGroupListLook {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
}

pub trait ControlGroupTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook;

    fn metrics(&self) -> MetricTokens;

    fn resolve_item_visual(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &StandardBoxScale,
        row_height: f32,
        row_radius: f32,
    ) -> ControlGroupItemVisualContext {
        let palette = self.default_item_palette(selected, state, size, scale);
        ControlGroupItemVisualContext {
            foreground: palette.foreground,
            background: palette.background,
            muted_foreground: palette.muted_foreground,
            typography: palette.typography,
            font_family: palette.font_family.clone(),
            adorner: palette.adorner,
            radius: row_radius,
            height: row_height.max(scale.height),
        }
    }

    fn resolve_item_adorner(&self, selected: bool, state: InteractionState, size: ControlSize) -> Option<AdornerSpec> {
        let scale = StandardBoxScale::compute(size, &self.metrics(), 1.0);
        self.default_item_palette(selected, state, size, &scale).adorner
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette;
}

#[derive(Clone, Debug)]
pub struct ControlGroupItemPalette {
    pub foreground: Hsla,
    pub background: Hsla,
    pub muted_foreground: Hsla,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub adorner: Option<AdornerSpec>,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultControlGroupTheme {
    tokens: ThemeTokens,
}

pub fn default_control_group_theme() -> Arc<dyn ControlGroupTheme> {
    static THEME: OnceLock<Arc<dyn ControlGroupTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultControlGroupTheme::default())).clone()
}

impl DefaultControlGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ControlGroupTheme for DefaultControlGroupTheme {
    fn resolve_list(&self, enabled: bool) -> ControlGroupListLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ControlGroupListLook {
            background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            border: palette.border.default,
            radius: metrics.radius(ControlSize::Md),
            padding_x: 6.0,
            padding_y: 4.0,
            gap: 6.0,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }

    fn default_item_palette(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> ControlGroupItemPalette {
        let button_theme = DefaultButtonFamilyTheme::new(self.tokens.clone());
        let role = ButtonFamilyRole::Toggle { selected };
        let palette = button_theme.resolve(role, size, state);
        let look = compose_button_family_look(&palette, role, scale, scale.radius);

        ControlGroupItemPalette {
            foreground: look.foreground,
            background: look.background,
            muted_foreground: self.tokens.palette.app.muted_foreground,
            typography: look.typography,
            font_family: look.font_family.clone(),
            adorner: look.adorner,
        }
    }
}
