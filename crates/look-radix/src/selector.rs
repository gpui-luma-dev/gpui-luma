//! Radix selector theme and builder backed by the shared SDK selector.
use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use gpui_luma::controls::selector::{
    SelectorBuilder, SelectorItem, SelectorItemLike, SelectorOpeningMode, SelectorPalette, SelectorPlacement,
    SelectorTemplate, SelectorTheme, SelectorTriggerStyle, ThemedSelectorTemplate,
};
use gpui_luma::controls::selector_list::{SelectorItemsPanelLook, default_selector_items_template};
use gpui_luma::theme::{InteractionState, MetricTokens};

use crate::{ButtonSize, Look, PopupMenuVariant, Tone};
use crate::look::resolve_look;

struct SelectorThemeAdapter {
    look: Look,
}

impl SelectorTheme for SelectorThemeAdapter {
    fn resolve(
        &self,
        style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        use gpui_luma::controls::popup_menu::{PopupMenuTriggerMetrics, PopupMenuTriggerStyle};
        let trigger_style = match style {
            SelectorTriggerStyle::Outline => PopupMenuTriggerStyle::Outline,
            SelectorTriggerStyle::Ghost => PopupMenuTriggerStyle::Ghost,
        };
        let palette = crate::popup_menu_theme(&self.look, PopupMenuVariant::Solid, Tone::Accent).resolve(
            trigger_style,
            PopupMenuTriggerMetrics::default(),
            state,
        );
        let panel = palette.floating_menu;
        SelectorPalette {
            trigger_background: palette.trigger_background,
            trigger_foreground: palette.trigger_foreground,
            trigger_icon: palette.trigger_foreground,
            trigger_border: palette.trigger_border,
            trigger_shadow: if without_elevation {
                None
            } else {
                palette.trigger_shadow
            },
            trigger_typography: palette.trigger_typography,
            items_panel: SelectorItemsPanelLook {
                background: panel.background,
                foreground: panel.foreground,
                border: panel.border,
                shadow: panel.shadow,
                radius: panel.radius,
                padding: panel.padding,
                min_width: panel.min_width,
                item_disabled_foreground: panel.item_disabled_foreground,
                item_hover_background: panel.item_hover_background,
                item_hover_foreground: panel.item_hover_foreground,
                item_typography: panel.item_typography,
                item_height: panel.item_height,
                item_padding_x: panel.item_padding_x,
                item_gap: panel.item_gap,
                item_icon_size: panel.item_icon_size,
                item_radius: panel.item_radius,
            },
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

/// Build a selector template using the Radix trigger and menu palette.
pub fn selector_template<T: SelectorItemLike + 'static>(look: &Look) -> Arc<dyn SelectorTemplate<T>> {
    Arc::new(ThemedSelectorTemplate::new(
        Arc::new(SelectorThemeAdapter { look: look.clone() }),
        default_selector_items_template(),
    ))
}

/// Radix selector builder. Opening and placement behavior are shared with the SDK.
pub struct Selector<T: SelectorItemLike + 'static = SelectorItem> {
    look: Option<Look>,
    builder: SelectorBuilder<T>,
}

impl Selector<SelectorItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id)
    }
}

impl<T: SelectorItemLike + 'static> Selector<T> {
    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self { look: None, builder: SelectorBuilder::new(id) }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.builder = self.builder.label(label);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selected_id(mut self, id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected_id(id);
        self
    }

    pub fn opening_mode(mut self, mode: SelectorOpeningMode) -> Self {
        self.builder = self.builder.opening_mode(mode);
        self
    }

    pub fn placement(mut self, placement: SelectorPlacement) -> Self {
        self.builder = self.builder.placement(placement);
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.builder = self.builder.size(size.control_size());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<gpui_luma::controls::selector::Selector<T>> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        self.builder.template(selector_template(&look)).spawn(cx)
    }
}
