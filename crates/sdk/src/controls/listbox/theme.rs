use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::adorner::{AdornerPlacement, AdornerSpec, FocusRingAdornerSpec};
use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, ListRowScale, LumaTextStyle, MetricTokens, ThemeTokens,
};

#[derive(Clone, Debug)]
pub struct ListBoxListLook {
    pub background: Hsla,
    pub border: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub divider: Hsla,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub row_gap: f32,
}

#[derive(Clone, Debug)]
pub struct ListBoxRowPalette {
    pub background: Hsla,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
}

#[derive(Clone, Debug)]
pub struct ListBoxRowLook {
    pub background: Hsla,
    pub label_color: Hsla,
    pub adorner: Option<AdornerSpec>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub height: f32,
    pub label_baseline_shift: f32,
}

pub trait ListBoxTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool, focused: bool, size: ControlSize) -> ListBoxListLook;
    fn resolve_row(&self, selected: bool, state: InteractionState, size: ControlSize) -> ListBoxRowPalette;
    fn metrics(&self) -> MetricTokens;

    fn resolve_row_look(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
        scale: &ListRowScale,
    ) -> ListBoxRowLook {
        compose_listbox_row_look(&self.resolve_row(selected, state, size), scale)
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultListBoxTheme {
    tokens: ThemeTokens,
}

pub fn default_listbox_theme() -> Arc<dyn ListBoxTheme> {
    static THEME: OnceLock<Arc<dyn ListBoxTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultListBoxTheme::default())).clone()
}

impl DefaultListBoxTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ListBoxTheme for DefaultListBoxTheme {
    fn resolve_list(&self, enabled: bool, focused: bool, size: ControlSize) -> ListBoxListLook {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        let adorner = if focused {
            Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
                color: palette.focus.ring,
                placement: AdornerPlacement::Inset,
                distance: metrics.border_width.default,
                width: metrics.focus.width,
            }))
        } else {
            None
        };

        ListBoxListLook {
            background: if enabled {
                palette.form.input.background
            } else {
                palette.state.disabled.background
            },
            border: palette.form.input.border,
            adorner,
            divider: palette.form.input.border,
            radius: metrics.radius(size),
            padding_x: 6.0,
            padding_y: metrics.padding_y(size) * 0.5,
            row_gap: metrics.padding_y(size) * 0.25,
        }
    }

    fn resolve_row(&self, _selected: bool, state: InteractionState, _size: ControlSize) -> ListBoxRowPalette {
        let palette = &self.tokens.palette;
        let typography = &self.tokens.typography;

        let transparent = Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };

        let background = match state.layer() {
            InteractionLayer::Disabled => transparent,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.hover.background,
            InteractionLayer::Default if state.focused => palette.state.hover.background,
            InteractionLayer::Default => transparent,
        };

        let label_color = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.app.foreground
        };

        let adorner = None;

        ListBoxRowPalette { background, label_color, adorner, label_typography: typography.text.label }
    }

    fn metrics(&self) -> MetricTokens {
        self.tokens.metrics
    }
}

pub(crate) fn compose_listbox_row_look(palette: &ListBoxRowPalette, scale: &ListRowScale) -> ListBoxRowLook {
    ListBoxRowLook {
        background: palette.background,
        label_color: palette.label_color,
        adorner: palette.adorner,
        label_typography: palette.label_typography,
        radius: scale.radius,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        height: scale.min_height,
        label_baseline_shift: scale.label_baseline_shift,
    }
}
