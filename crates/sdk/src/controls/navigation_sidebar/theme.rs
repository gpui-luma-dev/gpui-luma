use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{
    ControlSize, InteractionLayer, InteractionState, LumaTextStyle, ThemePartUsage, ThemeTokens, ThemeUsage,
};

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarContainerAppearance {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarSectionAppearance {
    pub label_color: Hsla,
    pub typography: LumaTextStyle,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct NavigationSidebarItemAppearance {
    pub background: Option<Hsla>,
    pub foreground: Hsla,
    pub icon_color: Hsla,
    pub focus_ring: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub icon_size: f32,
}

pub trait NavigationSidebarTheme: Send + Sync {
    fn resolve_container(&self) -> NavigationSidebarContainerAppearance;
    fn resolve_section(&self) -> NavigationSidebarSectionAppearance;
    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemAppearance;
    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> NavigationSidebarItemAppearance;

    fn usage(&self) -> &'static ThemeUsage {
        navigation_sidebar_theme_usage()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultNavigationSidebarTheme {
    tokens: ThemeTokens,
}

pub fn default_navigation_sidebar_theme() -> Arc<dyn NavigationSidebarTheme> {
    if let Some(radix) = crate::theme::radix::active_radix_theme() {
        return radix.navigation_sidebar_theme();
    }

    if let Some(live) = crate::theme::pack::active_live_theme() {
        return live;
    }
    static THEME: OnceLock<Arc<dyn NavigationSidebarTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultNavigationSidebarTheme::default())).clone()
}

pub fn navigation_sidebar_theme_usage() -> &'static ThemeUsage {
    &NAVIGATION_SIDEBAR_THEME_USAGE
}

pub const NAVIGATION_SIDEBAR_THEME_USAGE: ThemeUsage = ThemeUsage {
    label: "Navigation Sidebar",
    parts: &[
        ThemePartUsage {
            part: "container background",
            token: "navigation.background",
            states: &["default"],
            appearance_fields: &["NavigationSidebarContainerAppearance.background"],
        },
        ThemePartUsage {
            part: "container foreground",
            token: "navigation.foreground",
            states: &["default"],
            appearance_fields: &["NavigationSidebarContainerAppearance.foreground"],
        },
        ThemePartUsage {
            part: "container border",
            token: "navigation.border",
            states: &["default"],
            appearance_fields: &["NavigationSidebarContainerAppearance.border"],
        },
        ThemePartUsage {
            part: "section label",
            token: "navigation.muted_foreground",
            states: &["default"],
            appearance_fields: &["NavigationSidebarSectionAppearance.label_color"],
        },
        ThemePartUsage {
            part: "branch foreground",
            token: "navigation.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &[
                "NavigationSidebarItemAppearance.foreground",
                "NavigationSidebarItemAppearance.icon_color",
            ],
        },
        ThemePartUsage {
            part: "branch disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &[
                "NavigationSidebarItemAppearance.foreground",
                "NavigationSidebarItemAppearance.icon_color",
            ],
        },
        ThemePartUsage {
            part: "branch hover background",
            token: "navigation.hover_background",
            states: &["hovered"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "branch pressed background",
            token: "state.pressed.background",
            states: &["pressed"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "branch focus ring",
            token: "focus.ring",
            states: &["focused"],
            appearance_fields: &["NavigationSidebarItemAppearance.focus_ring"],
        },
        ThemePartUsage {
            part: "item foreground",
            token: "navigation.foreground",
            states: &["default", "hovered", "pressed", "focused"],
            appearance_fields: &[
                "NavigationSidebarItemAppearance.foreground",
                "NavigationSidebarItemAppearance.icon_color",
            ],
        },
        ThemePartUsage {
            part: "item disabled foreground",
            token: "state.disabled.foreground",
            states: &["disabled"],
            appearance_fields: &[
                "NavigationSidebarItemAppearance.foreground",
                "NavigationSidebarItemAppearance.icon_color",
            ],
        },
        ThemePartUsage {
            part: "item hover background",
            token: "navigation.hover_background",
            states: &["hovered"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "item pressed background",
            token: "state.pressed.background",
            states: &["pressed"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "selected item background",
            token: "navigation.selected_background",
            states: &["selected"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "selected item foreground",
            token: "navigation.selected_foreground",
            states: &["selected", "selected hovered", "selected pressed", "selected focused"],
            appearance_fields: &[
                "NavigationSidebarItemAppearance.foreground",
                "NavigationSidebarItemAppearance.icon_color",
            ],
        },
        ThemePartUsage {
            part: "selected item hover background",
            token: "action.prominent.hover_background",
            states: &["selected hovered"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "selected item pressed background",
            token: "action.prominent.pressed_background",
            states: &["selected pressed"],
            appearance_fields: &["NavigationSidebarItemAppearance.background"],
        },
        ThemePartUsage {
            part: "item focus ring",
            token: "focus.ring",
            states: &["focused", "selected focused"],
            appearance_fields: &["NavigationSidebarItemAppearance.focus_ring"],
        },
    ],
};

impl DefaultNavigationSidebarTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }

    fn base_item(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemAppearance {
        let palette = &self.tokens.palette;
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let size_metrics = metrics.for_size(size);
        let foreground = if state.disabled {
            palette.state.disabled.foreground
        } else {
            palette.navigation.foreground
        };

        NavigationSidebarItemAppearance {
            background: None,
            foreground,
            icon_color: foreground,
            focus_ring: state.focused.then_some(palette.focus.ring),
            typography: typography.text.label,
            radius: metrics.radius(size),
            height: 30.0,
            padding_x: 8.0,
            gap: size_metrics.gap,
            icon_size: 16.0,
        }
    }
}

impl NavigationSidebarTheme for DefaultNavigationSidebarTheme {
    fn resolve_container(&self) -> NavigationSidebarContainerAppearance {
        let navigation = &self.tokens.palette.navigation;

        NavigationSidebarContainerAppearance {
            background: navigation.background,
            foreground: navigation.foreground,
            border: navigation.border,
        }
    }

    fn resolve_section(&self) -> NavigationSidebarSectionAppearance {
        NavigationSidebarSectionAppearance {
            label_color: self.tokens.palette.navigation.muted_foreground,
            typography: self.tokens.typography.text.caption,
            height: 20.0,
        }
    }

    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> NavigationSidebarItemAppearance {
        let mut appearance = self.base_item(state, size);
        let palette = &self.tokens.palette;

        appearance.background = match state.layer() {
            InteractionLayer::Disabled | InteractionLayer::Default => None,
            InteractionLayer::Hovered => Some(palette.navigation.hover_background),
            InteractionLayer::Pressed => Some(palette.state.pressed.background),
        };

        appearance
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> NavigationSidebarItemAppearance {
        let mut appearance = self.base_item(state, size);
        let palette = &self.tokens.palette;

        appearance.background = match (selected, state.layer()) {
            (_, InteractionLayer::Disabled) => None,
            (true, InteractionLayer::Pressed) => Some(palette.action.prominent.pressed_background),
            (true, InteractionLayer::Hovered) => Some(palette.action.prominent.hover_background),
            (true, InteractionLayer::Default) => Some(palette.navigation.selected_background),
            (false, InteractionLayer::Pressed) => Some(palette.state.pressed.background),
            (false, InteractionLayer::Hovered) => Some(palette.navigation.hover_background),
            (false, InteractionLayer::Default) => None,
        };

        if selected && !state.disabled {
            appearance.foreground = palette.navigation.selected_foreground;
            appearance.icon_color = palette.navigation.selected_foreground;
        }

        appearance
    }
}

#[cfg(test)]
mod tests {
    use super::navigation_sidebar_theme_usage;

    #[test]
    fn usage_metadata_describes_navigation_sidebar_tokens() {
        let usage = navigation_sidebar_theme_usage();

        assert_eq!(usage.label, "Navigation Sidebar");
        assert!(usage.parts.iter().any(|part| part.token == "navigation.selected_background"));

        for part in usage.parts {
            assert!(!part.part.is_empty());
            assert!(!part.token.is_empty());
            assert!(!part.states.is_empty());
            assert!(!part.appearance_fields.is_empty());
        }
    }
}
