use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use super::{
    ButtonVariant, ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens,
    ThemeUsage,
};

#[derive(Clone, Copy, Debug)]
pub struct ChoiceGroupListAppearance {
    pub background: Hsla,
    pub border: Hsla,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ChoiceGroupItemAppearance {
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

pub trait ChoiceGroupTheme: Send + Sync {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> ChoiceGroupListAppearance;
    fn resolve_item(
        &self,
        variant: ButtonVariant,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> ChoiceGroupItemAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultChoiceGroupTheme {
    tokens: ThemeTokens,
}

pub fn default_choice_group_theme() -> Arc<dyn ChoiceGroupTheme> {
    static THEME: OnceLock<Arc<dyn ChoiceGroupTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultChoiceGroupTheme::default())).clone()
}

pub const CHOICE_GROUP_THEME_USAGE: ThemeUsage = ThemeUsage {
    component: "Choice Group",
    parts: &[
        ThemePartUsage {
            part: "list background",
            token: "surface.subtle.background",
            states: &["enabled"],
            appearance_fields: &["ChoiceGroupListAppearance.background"],
        },
        ThemePartUsage {
            part: "list border and item dividers",
            token: "border.default",
            states: &["default", "enabled", "disabled"],
            appearance_fields: &["ChoiceGroupListAppearance.border", "ChoiceGroupItemAppearance.divider"],
        },
        ThemePartUsage {
            part: "unselected item background",
            token: "surface.subtle.background",
            states: &["unselected"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected item hover background",
            token: "state.hover.background",
            states: &["unselected hovered"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected item pressed background",
            token: "state.pressed.background",
            states: &["unselected pressed", "selected disabled"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "default selected background",
            token: "state.selected.background",
            states: &["selected"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "default selected foreground",
            token: "state.selected.foreground",
            states: &["selected"],
            appearance_fields: &["ChoiceGroupItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "prominent selected background",
            token: "action.prominent.background",
            states: &["selected"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "prominent selected foreground",
            token: "action.prominent.foreground",
            states: &["selected"],
            appearance_fields: &["ChoiceGroupItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "prominent selected hover background",
            token: "action.prominent.hover_background",
            states: &["selected hovered"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "prominent selected pressed background",
            token: "action.prominent.pressed_background",
            states: &["selected pressed"],
            appearance_fields: &["ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "unselected label",
            token: "app.foreground",
            states: &["unselected"],
            appearance_fields: &["ChoiceGroupItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "disabled background",
            token: "state.disabled.background",
            states: &["disabled", "unselected disabled"],
            appearance_fields: &["ChoiceGroupListAppearance.background", "ChoiceGroupItemAppearance.background"],
        },
        ThemePartUsage {
            part: "disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &["ChoiceGroupItemAppearance.label_color"],
        },
        ThemePartUsage {
            part: "focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["ChoiceGroupItemAppearance.focus_ring"],
        },
    ],
};

impl DefaultChoiceGroupTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ChoiceGroupTheme for DefaultChoiceGroupTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> ChoiceGroupListAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;

        ChoiceGroupListAppearance {
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
    ) -> ChoiceGroupItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;

        let selected_background = match variant {
            ButtonVariant::Standard | ButtonVariant::Ghost => palette.state.selected.background,
            ButtonVariant::Prominent => palette.action.prominent.background,
        };
        let selected_hover = palette.action.prominent.hover_background;
        let selected_pressed = palette.action.prominent.pressed_background;

        let disabled_selected_background = palette.state.pressed.background;

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
                ButtonVariant::Standard | ButtonVariant::Ghost => palette.state.selected.foreground,
                ButtonVariant::Prominent => palette.action.prominent.foreground,
            },
            (_, false, false) => palette.app.foreground,
        };

        ChoiceGroupItemAppearance {
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
    use super::{ButtonVariant, ChoiceGroupTheme, ControlSize, DefaultChoiceGroupTheme, InteractionState};

    #[test]
    fn disabled_selected_items_keep_a_distinct_background() {
        let theme = DefaultChoiceGroupTheme::default();
        let disabled = InteractionState { disabled: true, ..Default::default() };

        let selected = theme.resolve_item(ButtonVariant::Standard, true, disabled, ControlSize::Md);
        let unselected = theme.resolve_item(ButtonVariant::Standard, false, disabled, ControlSize::Md);

        assert_ne!(selected.background, unselected.background);
        assert_eq!(selected.label_color, unselected.label_color);
    }
}
