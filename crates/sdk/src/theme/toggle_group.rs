use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ButtonVariant, ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemeTokens};

#[derive(Clone, Copy, Debug)]
pub struct ToggleGroupListAppearance {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ToggleGroupItemAppearance {
    pub background: Hsla,
    pub label_color: Hsla,
    pub divider: Hsla,
    pub focus_ring: Option<Hsla>,
    pub label_typography: LumaTextStyle,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub height: f32,
}

pub trait ToggleGroupTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> ToggleGroupListAppearance;
    fn resolve_item(
        &self,
        variant: ButtonVariant,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> ToggleGroupItemAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultToggleGroupTheme {
    tokens: ThemeTokens,
}

pub fn default_toggle_group_theme() -> Arc<dyn ToggleGroupTheme> {
    static THEME: OnceLock<Arc<dyn ToggleGroupTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultToggleGroupTheme::default())).clone()
}

impl DefaultToggleGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ToggleGroupTheme for DefaultToggleGroupTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> ToggleGroupListAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ToggleGroupListAppearance {
            background: if enabled {
                palette.surface.subtle.background
            } else {
                palette.state.disabled.background
            },
            border: palette.border.default,
            radius: metrics.radius(size),
        }
    }

    fn resolve_item(
        &self,
        variant: ButtonVariant,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> ToggleGroupItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;

        let selected_background = match variant {
            ButtonVariant::Default => palette.state.selected.background,
            ButtonVariant::Primary => palette.action.primary.background,
            ButtonVariant::Destructive => palette.action.danger.background,
        };
        let selected_hover = match variant {
            ButtonVariant::Default => palette.action.primary.hover_background,
            ButtonVariant::Primary => palette.action.primary.hover_background,
            ButtonVariant::Destructive => palette.action.danger.hover_background,
        };
        let selected_pressed = match variant {
            ButtonVariant::Default => palette.action.primary.pressed_background,
            ButtonVariant::Primary => palette.action.primary.pressed_background,
            ButtonVariant::Destructive => palette.action.danger.pressed_background,
        };

        let disabled_selected_background = match variant {
            ButtonVariant::Default => palette.state.pressed.background,
            ButtonVariant::Primary => palette.state.pressed.background,
            ButtonVariant::Destructive => palette.state.pressed.background,
        };

        let background = match (selected, state.layer()) {
            (true, InteractionLayer::Disabled) => disabled_selected_background,
            (false, InteractionLayer::Disabled) => palette.state.disabled.background,
            (true, InteractionLayer::Pressed) => selected_pressed,
            (true, InteractionLayer::Hovered) => selected_hover,
            (true, InteractionLayer::Default) => selected_background,
            (false, InteractionLayer::Pressed) => palette.state.pressed.background,
            (false, InteractionLayer::Hovered) => palette.state.hover.background,
            (false, InteractionLayer::Default) => palette.surface.subtle.background,
        };

        let label_color = match (variant, selected, state.disabled) {
            (_, _, true) => palette.state.disabled.foreground,
            (_, true, false) => match variant {
                ButtonVariant::Default => palette.state.selected.foreground,
                ButtonVariant::Primary => palette.action.primary.foreground,
                ButtonVariant::Destructive => palette.action.danger.foreground,
            },
            (_, false, false) => palette.app.foreground,
        };

        ToggleGroupItemAppearance {
            background,
            label_color,
            divider: palette.border.default,
            focus_ring: state.focused.then_some(palette.focus.ring),
            label_typography: typography.text.label,
            radius: metrics.radius(size),
            padding_x: metrics.padding_x(size),
            padding_y: metrics.padding_y(size),
            height: metrics.control_height(size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonVariant, ControlSize, DefaultToggleGroupTheme, InteractionState, ToggleGroupTheme};

    #[test]
    fn disabled_selected_items_keep_a_distinct_background() {
        let theme = DefaultToggleGroupTheme::default();
        let disabled = InteractionState { disabled: true, ..Default::default() };

        let selected = theme.resolve_item(ButtonVariant::Default, true, disabled, ControlSize::Md);
        let unselected = theme.resolve_item(ButtonVariant::Default, false, disabled, ControlSize::Md);

        assert_ne!(selected.background, unselected.background);
        assert_eq!(selected.label_color, unselected.label_color);
    }
}
