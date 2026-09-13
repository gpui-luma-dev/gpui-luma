//! Radix tabs theme adapter.

use std::sync::Arc;

use gpui::{FontWeight, SharedString};
use luma::controls::tabs::{TabsItemLook, TabsListLook, TabsTemplate, TabsTheme, ThemedTabsTemplate};
use luma::theme::{ControlSize, InteractionState, LumaTextStyle};

use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::typography::font_family;

/// Radix `size` prop on Tabs. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsSize {
    One,
    #[default]
    Two,
}

impl TabsSize {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
        }
    }

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
        }
    }
}

/// Radix Themes `variant` on Tabs. Default is [`Self::Line`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsVariant {
    /// Underline indicator, no list box.
    #[default]
    Line,
    /// Padded gray list with a selected chip. No underline.
    Surface,
}

struct TabsThemeAdapter {
    look: Look,
    variant: TabsVariant,
}

impl TabsTheme for TabsThemeAdapter {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        let metrics = self.look.metrics();
        match self.variant {
            TabsVariant::Line => TabsListLook {
                // Underline tabs: no list box border. Style-guide / sidebar chrome draws its own
                // full-width hairline under the control (see luma-studio accordion / buttons sections).
                background: (!enabled).then(|| self.look.resolve_role(SemanticRole::Surface).hsla()),
                border: None,
                radius: 0.0,
                padding: 0.0,
                gap: metrics.gap(size),
            },
            TabsVariant::Surface => TabsListLook {
                background: Some(self.look.resolve_step(ScaleFamily::Gray, 3).hsla()),
                border: None,
                radius: metrics.radius(size),
                padding: metrics.spacing.s1,
                gap: 2.0,
            },
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let metrics = self.look.metrics();
        let control = metrics.for_size(size);
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        let fg = self.look.resolve_role(SemanticRole::Foreground).hsla();
        let accent = self.look.resolve_role(SemanticRole::Primary).hsla();
        let surface = matches!(self.variant, TabsVariant::Surface);

        let label_color = if state.disabled {
            muted
        } else if active {
            fg
        } else if state.hovered {
            fg
        } else {
            muted
        };

        TabsItemLook {
            label_color,
            indicator: (!surface && active).then_some(accent),
            background: (surface && active).then(|| self.look.resolve_role(SemanticRole::Background).hsla()),
            label_typography: LumaTextStyle {
                size: match size {
                    ControlSize::Sm => 12.5,
                    ControlSize::Md => 14.0,
                    ControlSize::Lg => 16.0,
                },
                line_height: match size {
                    ControlSize::Sm => 18.0,
                    ControlSize::Md => 20.0,
                    ControlSize::Lg => 22.0,
                },
                weight: if active { FontWeight::MEDIUM } else { FontWeight::NORMAL },
            },
            radius: if surface { metrics.radius(size) } else { 0.0 },
            padding_x: control.padding_x,
            height: control.height * 0.85,
            indicator_height: if surface { 0.0 } else { 2.0 },
        }
    }

    fn font_family(&self) -> SharedString {
        font_family(&self.look)
    }
}

pub fn tabs_theme(look: &Look) -> Arc<dyn TabsTheme> {
    tabs_theme_for(look, TabsVariant::Line)
}

pub fn tabs_theme_for(look: &Look, variant: TabsVariant) -> Arc<dyn TabsTheme> {
    Arc::new(TabsThemeAdapter { look: look.clone(), variant })
}

pub fn tabs_template(look: &Look) -> Arc<dyn TabsTemplate> {
    tabs_template_for(look, TabsVariant::Line)
}

pub fn tabs_template_for(look: &Look, variant: TabsVariant) -> Arc<dyn TabsTemplate> {
    Arc::new(ThemedTabsTemplate::new(tabs_theme_for(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::ControlSize;

    #[test]
    fn line_tabs_use_underline_indicator() {
        let look = Look::built_in();
        let theme = tabs_theme(&look);
        let list = theme.resolve_list(true, ControlSize::Md);
        let active = theme.resolve_item(true, InteractionState::default(), ControlSize::Md);
        let inactive = theme.resolve_item(false, InteractionState::default(), ControlSize::Md);

        assert!(list.background.is_none());
        assert_eq!(list.padding, 0.0);
        assert_eq!(list.radius, 0.0);
        assert!(active.indicator.is_some());
        assert!(active.background.is_none());
        assert!(inactive.indicator.is_none());
        assert_eq!(active.radius, 0.0);
    }

    #[test]
    fn surface_tabs_use_list_fill_and_selected_chip() {
        let look = Look::built_in();
        let theme = tabs_theme_for(&look, TabsVariant::Surface);
        let list = theme.resolve_list(true, ControlSize::Md);
        let active = theme.resolve_item(true, InteractionState::default(), ControlSize::Md);
        let inactive = theme.resolve_item(false, InteractionState::default(), ControlSize::Md);

        assert!(list.background.is_some());
        assert!(list.padding > 0.0);
        assert!(list.radius > 0.0);
        assert!(active.indicator.is_none());
        assert!(inactive.indicator.is_none());
        assert_eq!(active.background, Some(look.resolve_role(SemanticRole::Background).hsla()));
        assert!(inactive.background.is_none());
        assert!(active.radius > 0.0);
    }
}
