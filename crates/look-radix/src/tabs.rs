//! Radix tabs theme adapter.

use std::sync::Arc;

use gpui::{FontWeight, Hsla, SharedString};
use gpui_luma::controls::tabs::{TabsBaseline, TabsItemLook, TabsListLook, TabsTemplate, TabsTheme, ThemedTabsTemplate};
use gpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle};

use crate::look::Look;
use crate::button::{ButtonVariant, Paint, button_family_theme_with};
use gpui_luma::controls::button_family::ButtonFamilyRole;
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
    /// Underline indicator with a full-width baseline, no list box.
    #[default]
    Line,
    /// Padded gray list with a selected chip. No underline.
    Surface,
}

/// Geometry for one Radix tab size, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsMetrics {
    pub height: f32,
    /// Total horizontal trigger padding (outer + inner padding in Radix CSS).
    pub padding_x: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// Full-control baseline, independent of the active-tab indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsBaselineStyle {
    pub height: f32,
    pub inset_x: f32,
    pub bottom: f32,
    /// Neutral scale step. The current catalog uses solid colors, so step 5
    /// approximates Radix's gray-a5 rule on the default page background.
    pub gray_step: u8,
    /// Optional exact color; otherwise resolve `gray_step` in the current mode.
    pub color: Option<Hsla>,
}

impl Default for TabsBaselineStyle {
    fn default() -> Self {
        Self { height: 1.0, inset_x: 0.0, bottom: 0.0, gray_step: 5, color: None }
    }
}

/// Look-wide tab tokens. The baseline and selected indicator can be tuned separately.
///
/// ```
/// let look = gpui_luma_look_radix::Look::built_in();
/// let mut style = look.tabs_style();
/// if let Some(baseline) = &mut style.baseline {
///     baseline.height = 2.0;
///     baseline.inset_x = 4.0;
/// }
/// look.set_tabs_style(style);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsStyle {
    /// `None` hides only the full-list baseline.
    pub baseline: Option<TabsBaselineStyle>,
    pub indicator_height: f32,
    /// Zero spans the whole tab, independently of label padding.
    pub indicator_inset: f32,
    pub gap: f32,
    pub small: TabsMetrics,
    pub medium: TabsMetrics,
}

impl Default for TabsStyle {
    fn default() -> Self {
        Self {
            baseline: Some(TabsBaselineStyle::default()),
            indicator_height: 2.0,
            indicator_inset: 0.0,
            gap: 0.0,
            small: TabsMetrics { height: 32.0, padding_x: 8.0, font_size: 12.0, line_height: 16.0 },
            medium: TabsMetrics { height: 40.0, padding_x: 16.0, font_size: 14.0, line_height: 20.0 },
        }
    }
}

impl TabsStyle {
    fn metrics(self, size: ControlSize) -> TabsMetrics {
        match size {
            ControlSize::Sm => self.small,
            ControlSize::Md | ControlSize::Lg => self.medium,
        }
    }
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
                // The SDK paints the separate baseline without adding layout borders.
                background: (!enabled).then(|| self.look.resolve_role(SemanticRole::Surface).hsla()),
                border: None,
                radius: 0.0,
                padding: 0.0,
                gap: self.look.tabs_style().gap,
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
        let style = self.look.tabs_style();
        let tab = style.metrics(size);
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

        let background = if surface && active {
            Some(self.look.resolve_role(SemanticRole::Background).hsla())
        } else if !state.disabled && (state.hovered || state.pressed) {
            Some(
                button_family_theme_with(&self.look, ButtonVariant::Ghost, Paint::gray())
                    .resolve(ButtonFamilyRole::Text, size, state)
                    .background,
            )
        } else {
            None
        };
        TabsItemLook {
            label_color,
            indicator: (!surface && active).then_some(accent),
            background,
            label_typography: LumaTextStyle {
                size: tab.font_size,
                line_height: tab.line_height,
                weight: if active { FontWeight::MEDIUM } else { FontWeight::NORMAL },
            },
            radius: metrics.radius(size),
            padding_x: tab.padding_x,
            height: tab.height,
            indicator_height: if surface { 0.0 } else { style.indicator_height },
        }
    }

    fn baseline(&self, _enabled: bool, _size: ControlSize) -> Option<TabsBaseline> {
        if self.variant != TabsVariant::Line {
            return None;
        }
        self.look.tabs_style().baseline.map(|baseline| TabsBaseline {
            color: baseline
                .color
                .unwrap_or_else(|| self.look.resolve_step(ScaleFamily::Gray, baseline.gray_step).hsla()),
            height: baseline.height,
            inset_x: baseline.inset_x,
            bottom: baseline.bottom,
        })
    }

    fn indicator_inset(&self, _size: ControlSize) -> Option<f32> {
        Some(self.look.tabs_style().indicator_inset.max(0.0))
    }

    fn content_padding(&self, size: ControlSize) -> Option<(f32, f32)> {
        (self.variant == TabsVariant::Line).then_some(match size {
            ControlSize::Sm => (4.0, 2.0),
            ControlSize::Md | ControlSize::Lg => (8.0, 4.0),
        })
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
    use gpui_luma::theme::ControlSize;

    #[test]
    fn line_hover_and_press_use_gray_ghost_palette() {
        let look = Look::built_in();
        let tabs = tabs_theme(&look);
        let ghost = button_family_theme_with(&look, ButtonVariant::Ghost, Paint::gray());
        for mode in [gpui_luma::theme::ThemeMode::Light, gpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for state in [
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { hovered: true, pressed: true, ..Default::default() },
            ] {
                for active in [false, true] {
                    let item = tabs.resolve_item(active, state, ControlSize::Md);
                    assert_eq!(
                        item.background,
                        Some(ghost.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background)
                    );
                    let resting = tabs.resolve_item(active, InteractionState::default(), ControlSize::Md);
                    assert_eq!(item.indicator, resting.indicator);
                    assert_eq!(item.padding_x, resting.padding_x);
                    assert_eq!(item.height, resting.height);
                    assert!(resting.background.is_none());
                    assert!(
                        tabs.resolve_item(active, InteractionState { disabled: true, ..state }, ControlSize::Md)
                            .background
                            .is_none()
                    );
                }
            }
        }
        assert_eq!(tabs.content_padding(ControlSize::Sm), Some((4.0, 2.0)));
        assert_eq!(tabs.content_padding(ControlSize::Md), Some((8.0, 4.0)));
    }

    #[test]
    fn baseline_can_change_without_changing_tab_geometry_or_indicator() {
        let look = Look::built_in();
        let theme = tabs_theme(&look);
        let original = theme.resolve_item(true, InteractionState::default(), ControlSize::Md);
        let mut style = look.tabs_style();
        style.baseline = Some(TabsBaselineStyle { height: 3.0, inset_x: 12.0, bottom: 4.0, gray_step: 8, color: None });
        look.set_tabs_style(style);
        for mode in [gpui_luma::theme::ThemeMode::Light, gpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            let baseline = theme.baseline(true, ControlSize::Md).unwrap();
            assert_eq!((baseline.height, baseline.inset_x, baseline.bottom), (3.0, 12.0, 4.0));
            assert_eq!(baseline.color, look.resolve_step(ScaleFamily::Gray, 8).hsla());
            let active = theme.resolve_item(true, InteractionState::default(), ControlSize::Md);
            assert_eq!(active.height, original.height);
            assert_eq!(active.padding_x, original.padding_x);
            assert_eq!(active.indicator_height, original.indicator_height);
            assert_eq!(theme.indicator_inset(ControlSize::Md), Some(0.0));
        }
        style.baseline = None;
        look.set_tabs_style(style);
        assert!(theme.baseline(true, ControlSize::Md).is_none());
        assert!(theme.resolve_item(true, InteractionState::default(), ControlSize::Md).indicator.is_some());
    }

    #[test]
    fn fork_copies_tab_style_and_keeps_edits_independent() {
        let look = Look::built_in();
        let mut style = look.tabs_style();
        style.gap = 4.0;
        style.small.height = 36.0;
        look.set_tabs_style(style);
        let fork = look.fork();
        assert_eq!(fork.tabs_style(), style);
        style.indicator_inset = 8.0;
        fork.set_tabs_style(style);
        assert_eq!(look.tabs_style().indicator_inset, 0.0);
        assert_eq!(tabs_theme(&fork).indicator_inset(ControlSize::Sm), Some(8.0));
    }

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
        assert!(active.radius > 0.0);
        assert_eq!(active.height, 40.0);
        assert_eq!(active.padding_x, 16.0);
        assert_eq!(list.gap, 0.0);
        assert_eq!(theme.baseline(true, ControlSize::Md).unwrap().height, 1.0);
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
        assert!(theme.baseline(true, ControlSize::Md).is_none());
    }
}
