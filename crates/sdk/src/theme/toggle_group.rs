use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{ButtonVariant, ControlSize, InteractionLayer, InteractionState, ThemeTokens};

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
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;

        ToggleGroupListAppearance {
            background: if enabled {
                colors.surface
            } else {
                colors.surface_disabled
            },
            border: colors.border,
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
        let colors = &self.tokens.colors;
        let metrics = &self.tokens.metrics;

        let selected_background = match variant {
            ButtonVariant::Default => colors.surface_pressed,
            ButtonVariant::Primary => colors.primary,
            ButtonVariant::Destructive => colors.destructive,
        };
        let selected_hover = match variant {
            ButtonVariant::Default => colors.surface_pressed,
            ButtonVariant::Primary => colors.primary_hover,
            ButtonVariant::Destructive => colors.destructive_hover,
        };
        let selected_pressed = match variant {
            ButtonVariant::Default => colors.surface_pressed,
            ButtonVariant::Primary => colors.primary_pressed,
            ButtonVariant::Destructive => colors.destructive_pressed,
        };

        let disabled_selected_background = match variant {
            ButtonVariant::Default => colors.surface_pressed,
            ButtonVariant::Primary => colors.primary_pressed,
            ButtonVariant::Destructive => colors.destructive_pressed,
        };

        let background = match (selected, state.layer()) {
            (true, InteractionLayer::Disabled) => disabled_selected_background,
            (false, InteractionLayer::Disabled) => colors.surface_disabled,
            (true, InteractionLayer::Pressed) => selected_pressed,
            (true, InteractionLayer::Hovered) => selected_hover,
            (true, InteractionLayer::Default) => selected_background,
            (false, InteractionLayer::Pressed) => colors.surface_pressed,
            (false, InteractionLayer::Hovered) => colors.surface_hover,
            (false, InteractionLayer::Default) => colors.surface,
        };

        let label_color = match (variant, selected, state.disabled) {
            (_, _, true) => colors.text_disabled,
            (ButtonVariant::Default, _, false) => colors.text,
            (_, true, false) => colors.text_inverse,
            (_, false, false) => colors.text,
        };

        ToggleGroupItemAppearance {
            background,
            label_color,
            divider: colors.border,
            focus_ring: state.focused.then_some(colors.focus_ring),
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
