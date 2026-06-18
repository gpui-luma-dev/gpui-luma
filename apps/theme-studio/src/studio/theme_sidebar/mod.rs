mod accordion;
mod controls;
mod model;
mod panels;
mod parsing;
mod subscriptions;
mod sync;

use std::collections::{HashMap, HashSet};

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::accordion::AccordionControl;
use gpui_luma::controls::selector::Selector;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use self::controls::{
    TokenFieldBuilderExt, build_metric_field, build_metric_slider, build_shadow_color_field, build_shadow_number_field,
    build_shadow_slider, theme_selector_items,
};
use self::model::{SidebarTab, ThemeSidebarViewModel, TOKEN_CATEGORIES};
use self::panels::{render_colors_panel, render_other_panel, render_typography_panel};
use self::parsing::{effective_radius_rem, effective_spacing_rem, token_color_with_fallback, token_hex_value};
use super::content_tabs::theme_studio_tabs_navigation_template;
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::{
    RADIUS_REM_MAX, RADIUS_REM_MIN, SHADOW_BLUR_MAX, SHADOW_BLUR_MIN, SHADOW_OFFSET_X_MAX, SHADOW_OFFSET_X_MIN,
    SHADOW_OFFSET_Y_MAX, SHADOW_OFFSET_Y_MIN, SHADOW_OPACITY_MAX, SHADOW_OPACITY_MIN, SHADOW_SPREAD_MAX,
    SHADOW_SPREAD_MIN, SPACING_REM_MAX, SPACING_REM_MIN, StudioOverrides, resolved_shadow_override,
};

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

        let mut token_fields = HashMap::new();
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let initial = token_hex_value(&look, &global_overrides, token);
                let field = look
                    .textfield(format!("theme-studio-token-{token}"))
                    .value(initial)
                    .full_width(true)
                    .token_style()
                    .spawn(cx);
                token_fields.insert(token.to_string(), field);
            }
        }

        let radius_rem = effective_radius_rem(&look, overrides);
        let spacing_rem = effective_spacing_rem(&look, overrides);
        let radius_field = build_metric_field(&look, "radius", radius_rem, cx);
        let spacing_field = build_metric_field(&look, "spacing", spacing_rem, cx);
        let radius_slider = build_metric_slider(&look, "radius", RADIUS_REM_MIN, RADIUS_REM_MAX, radius_rem, cx);
        let spacing_slider = build_metric_slider(&look, "spacing", SPACING_REM_MIN, SPACING_REM_MAX, spacing_rem, cx);

        let shadow = resolved_shadow_override(&look, overrides);
        let shadow_color_field = build_shadow_color_field(&look, &shadow, cx);
        let shadow_opacity_field = build_shadow_number_field(&look, "shadow-opacity", shadow.opacity(), cx);
        let shadow_blur_field = build_shadow_number_field(&look, "shadow-blur", shadow.blur_px, cx);
        let shadow_spread_field = build_shadow_number_field(&look, "shadow-spread", shadow.spread_px, cx);
        let shadow_offset_x_field = build_shadow_number_field(&look, "shadow-offset-x", shadow.offset_x_px, cx);
        let shadow_offset_y_field = build_shadow_number_field(&look, "shadow-offset-y", shadow.offset_y_px, cx);
        let shadow_opacity_slider =
            build_shadow_slider(&look, "shadow-opacity", SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX, shadow.opacity(), cx);
        let shadow_blur_slider =
            build_shadow_slider(&look, "shadow-blur", SHADOW_BLUR_MIN, SHADOW_BLUR_MAX, shadow.blur_px, cx);
        let shadow_spread_slider =
            build_shadow_slider(&look, "shadow-spread", SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX, shadow.spread_px, cx);
        let shadow_offset_x_slider = build_shadow_slider(
            &look,
            "shadow-offset-x",
            SHADOW_OFFSET_X_MIN,
            SHADOW_OFFSET_X_MAX,
            shadow.offset_x_px,
            cx,
        );
        let shadow_offset_y_slider = build_shadow_slider(
            &look,
            "shadow-offset-y",
            SHADOW_OFFSET_Y_MIN,
            SHADOW_OFFSET_Y_MAX,
            shadow.offset_y_px,
            cx,
        );

        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);
        let other_accordion = Self::build_other_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self {
            vm: ThemeSidebarViewModel {
                look,
                global_overrides,
                token_fields,
                radius_field,
                spacing_field,
                radius_slider,
                spacing_slider,
                shadow_override: shadow.clone(),
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
