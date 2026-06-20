mod accordion;
mod model;
mod panels;
mod parsing;
mod subscriptions;
mod sync;

use std::collections::HashSet;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::AccordionControl;
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use self::model::{SidebarTab, ThemeSidebarViewModel, TOKEN_CATEGORIES};
use self::panels::{
    OtherPanelControls, build_other_panel_controls, build_token_fields, render_colors_panel, render_other_panel,
    render_typography_panel,
};
use self::parsing::token_color_with_fallback;
use super::content_tabs::theme_studio_tabs_navigation_template;
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::{StudioOverrides};
use crate::theme::available_themes;

pub struct ThemeSidebar {
    vm: ThemeSidebarViewModel,
    theme_selector: Entity<Selector>,
    tabs: Entity<TabsNavigation>,
    active_tab: SidebarTab,
    pub(super) token_accordion: Entity<AccordionControl>,
    pub(super) other_accordion: Entity<AccordionControl>,
    _subscriptions: Vec<Subscription>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<ThemeStudioApp>,
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let theme_items = theme_selector_items();
        let theme_selector = look
            .selector("theme-studio-theme-selector")
            .label("Theme")
            .items(theme_items)
            .selected_id(active_theme_id.clone())
            .spawn(cx);

        let tabs = look
            .tabs_navigation("theme-studio-sidebar-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(theme_studio_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("colors").label("Colors"),
                TabsNavigationItem::new("typography").label("Typography"),
                TabsNavigationItem::new("other").label("Other"),
            ])
            .active("colors")
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |sidebar, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = SidebarTab::from_id(tab_id.as_ref()) {
                sidebar.active_tab = tab;
                cx.notify();
            }
        }));

        let token_fields = build_token_fields(&look, &global_overrides, cx);
        let OtherPanelControls {
            palette_hsl,
            palette_hs,
            palette_hue_field,
            palette_saturation_field,
            palette_lightness_field,
            palette_hue_slider,
            palette_saturation_slider,
            palette_lightness_slider,
            palette_vividness_slider,
            palette_temperature_slider,
            radius_field,
            spacing_field,
            radius_slider,
            spacing_slider,
            shadow_override,
            shadow_color_field,
            shadow_opacity_field,
            shadow_blur_field,
            shadow_spread_field,
            shadow_offset_x_field,
            shadow_offset_y_field,
            shadow_opacity_slider,
            shadow_blur_slider,
            shadow_spread_slider,
            shadow_offset_x_slider,
            shadow_offset_y_slider,
        } = build_other_panel_controls(&look, overrides, cx);

        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);
        let other_accordion = Self::build_other_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self {
            vm: ThemeSidebarViewModel {
                look,
                global_overrides,
                palette_hsl,
                palette_hs,
                token_fields,
                palette_hue_field,
                palette_saturation_field,
                palette_lightness_field,
                palette_hue_slider,
                palette_saturation_slider,
                palette_lightness_slider,
                palette_vividness_slider,
                palette_temperature_slider,
                radius_field,
                spacing_field,
                radius_slider,
                spacing_slider,
                shadow_override,
                shadow_color_field,
                shadow_opacity_field,
                shadow_blur_field,
                shadow_spread_field,
                shadow_offset_x_field,
                shadow_offset_y_field,
                shadow_opacity_slider,
                shadow_blur_slider,
                shadow_spread_slider,
                shadow_offset_x_slider,
                shadow_offset_y_slider,
            },
            theme_selector,
            tabs,
            active_tab: SidebarTab::Colors,
            token_accordion,
            other_accordion,
            _subscriptions: subscriptions,
        }
    }
}

fn theme_selector_items() -> Vec<SelectorItem> {
    let mut items = vec![SelectorItem::new("default").label("Default")];
    for theme in available_themes() {
        items.push(SelectorItem::new(theme.id).label(theme.display_name()));
    }
    items
}

pub(crate) fn palette_tokens() -> Vec<&'static str> {
    TOKEN_CATEGORIES.iter().flat_map(|(_, tokens)| tokens.iter().map(|(token, _)| *token)).collect()
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.vm.look.chrome();
        let sidebar_bg =
            token_color_with_fallback(&self.vm.look, &self.vm.global_overrides, "sidebar", chrome.panel_background);

        let tab_body = match self.active_tab {
            SidebarTab::Colors => render_colors_panel(self),
            SidebarTab::Typography => render_typography_panel(),
            SidebarTab::Other => render_other_panel(self),
        };

        div()
            .id("theme-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("theme-studio-sidebar-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(self.theme_selector.clone()),
            )
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(
                div()
                    .id("theme-studio-sidebar-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_y_scroll()
                    .p(px(12.0))
                    .child(tab_body),
            )
    }
}
