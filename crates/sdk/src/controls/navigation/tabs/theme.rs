use std::sync::{Arc, OnceLock};

use gpui::{Hsla, SharedString};

use crate::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct TabsListLook {
    /// List border thickness in pixels.
    pub border_width: f32,
    pub background: Option<Hsla>,
    pub border: Option<Hsla>,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TabsItemLook {
    /// Accessory glyph size and gap in pixels.
    pub accessory_size: f32,
    pub accessory_gap: f32,
    pub label_color: Hsla,
    pub indicator: Option<Hsla>,
    /// Selected-chip fill. `None` is transparent (line / underline tabs).
    pub background: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub height: f32,
    pub indicator_height: f32,
}

/// Full-list rule painted behind the active indicator without consuming layout space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsBaseline {
    pub color: Hsla,
    pub height: f32,
    pub inset_x: f32,
    /// Distance above the bottom of the list, in logical pixels.
    pub bottom: f32,
}

pub trait TabsTheme: Send + Sync {
    /// Appearance of native drag previews and insertion feedback.
    fn resolve_drag(&self, size: ControlSize) -> crate::infra::drag_drop::DragDropLook {
        let mut drag = crate::infra::drag_drop::DragDropLook::from_tokens(&ThemeTokens::default(), size);
        let list = self.resolve_list(true, size);
        let item = self.resolve_item(true, Default::default(), size);
        drag.background = list.background.or(item.background).unwrap_or(drag.background);
        drag.foreground = item.label_color;
        drag.valid_marker = item.indicator.unwrap_or(item.label_color);
        drag.invalid_marker =
            self.resolve_item(false, InteractionState { invalid: true, ..Default::default() }, size).label_color;
        drag.radius = list.radius;
        drag.typography = item.label_typography;
        drag.font_family = self.font_family();
        drag
    }
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook;
    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook;
    fn font_family(&self) -> SharedString;
    /// Optional horizontal/vertical padding for an inset label background.
    /// Horizontal padding is taken from the trigger padding, preserving its width.
    fn content_padding(&self, _size: ControlSize) -> Option<(f32, f32)> {
        None
    }
    /// Independent of the selected-tab indicator. Absent by default.
    fn baseline(&self, _enabled: bool, _size: ControlSize) -> Option<TabsBaseline> {
        None
    }
    /// Override the active indicator's inset; `None` retains the item padding inset.
    fn indicator_inset(&self, _size: ControlSize) -> Option<f32> {
        None
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultTabsTheme {
    tokens: ThemeTokens,
}

pub fn default_tabs_theme() -> Arc<dyn TabsTheme> {
    static THEME: OnceLock<Arc<dyn TabsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultTabsTheme::default())).clone()
}

impl DefaultTabsTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl TabsTheme for DefaultTabsTheme {
    fn resolve_drag(&self, size: ControlSize) -> crate::infra::drag_drop::DragDropLook {
        crate::infra::drag_drop::DragDropLook::from_tokens(&self.tokens, size)
    }
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        let palette = self.tokens.stylesheet.recipes.palette("tabs", &self.tokens.palette);
        let metrics = &self.tokens.metrics;

        TabsListLook {
            border_width: metrics.border_width.default,
            background: (!enabled).then_some(palette.state.disabled.background),
            border: None,
            radius: metrics.radius(size),
            padding: self.tokens.stylesheet.tabs.geometry.list_padding.unwrap_or_default(),
            gap: self.tokens.stylesheet.tabs.geometry.list_gap.unwrap_or_else(|| metrics.gap(size)),
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let palette = self.tokens.stylesheet.recipes.palette("tabs", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let layer = state.layer();
        let label_typography = match size {
            ControlSize::Sm => typography.text.caption,
            ControlSize::Md => typography.text.label,
            ControlSize::Lg => typography.text.body,
        };

        let active_color = match layer {
            InteractionLayer::Disabled => palette.state.disabled.foreground,
            InteractionLayer::Pressed => palette.state.pressed.background,
            InteractionLayer::Hovered => palette.state.selected.background,
            InteractionLayer::Default => palette.state.selected.background,
        };

        let (accessory_size, accessory_gap) = self.tokens.stylesheet.tabs.accessory_metrics("");
        TabsItemLook {
            accessory_size,
            accessory_gap,
            label_color: match (active, state.disabled) {
                (_, true) => palette.state.disabled.foreground,
                (true, false) => active_color,
                (false, false) => palette.app.foreground,
            },
            indicator: active.then_some(if state.focused && !state.disabled {
                palette.focus.ring
            } else {
                active_color
            }),
            background: None,
            label_typography,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            height: label_typography.line_height + metrics.padding_y(size) * 2.0,
            indicator_height: self.tokens.stylesheet.tabs.geometry.indicator_height.unwrap_or_default(),
        }
    }

    fn font_family(&self) -> SharedString {
        self.tokens.typography.font.sans.family.clone().into()
    }
}

#[cfg(test)]
mod state_audit_tests {
    use super::*;

    #[test]
    fn disabled_active_tab_does_not_paint_a_focus_indicator() {
        let tokens = ThemeTokens::default();
        let theme = DefaultTabsTheme::new(tokens.clone());
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let look = theme.resolve_item(
                true,
                InteractionState { disabled: true, focused: true, hovered: true, pressed: true, ..Default::default() },
                size,
            );
            assert_eq!(look.label_color, tokens.palette.state.disabled.foreground);
            assert_eq!(look.indicator, Some(tokens.palette.state.disabled.foreground));
        }
    }
}
